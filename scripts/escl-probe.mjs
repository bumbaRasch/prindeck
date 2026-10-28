#!/usr/bin/env node

import { mkdir, writeFile } from 'node:fs/promises';
import { basename, join } from 'node:path';
import { parseArgs } from 'node:util';

const HOST = process.env.PRINDECK_HOST ?? '192.0.2.10';
const BASE = `http://${HOST}/eSCL`;
const DOC_TIMEOUT_MS = 120_000;

const tag = (xml, name) => xml.match(new RegExp(`<${name}>([^<]*)</${name}>`))?.[1];
const tags = (xml, name) => [...xml.matchAll(new RegExp(`<${name}>([^<]*)</${name}>`, 'g'))].map((m) => m[1]);

async function request(path, { method = 'GET', body, timeout = 20_000 } = {}) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: body ? { 'Content-Type': 'text/xml' } : undefined,
    body,
    signal: AbortSignal.timeout(timeout),
  });
  const buf = Buffer.from(await res.arrayBuffer());
  return { status: res.status, headers: Object.fromEntries(res.headers), body: buf, text: buf.toString('utf8') };
}

function parseScannerStatus(statusXml) {
  const jobs = [];
  const blocks = statusXml.match(/<scan:JobInfo>[\s\S]*?<\/scan:JobInfo>/g) ?? [];
  for (const block of blocks) {
    jobs.push({
      jobUri: tag(block, 'pwg:JobUri'),
      jobUuid: tag(block, 'pwg:JobUuid'),
      age: Number(tag(block, 'scan:Age')),
      imagesCompleted: Number(tag(block, 'pwg:ImagesCompleted')),
      imagesToTransfer: Number(tag(block, 'pwg:ImagesToTransfer')),
      state: tag(block, 'pwg:JobState'),
      stateReasons: tags(block, 'pwg:JobStateReason'),
    });
  }
  return {
    version: tag(statusXml, 'pwg:Version'),
    state: tag(statusXml, 'pwg:State'),
    adfState: tag(statusXml, 'scan:AdfState'),
    jobs,
  };
}

async function getCapabilities() {
  const res = await request('/ScannerCapabilities');
  if (res.status !== 200) throw new Error(`ScannerCapabilities: HTTP ${res.status}`);
  return res.text;
}

async function getStatus() {
  const res = await request('/ScannerStatus');
  if (res.status !== 200) throw new Error(`ScannerStatus: HTTP ${res.status}`);
  return { xml: res.text, parsed: parseScannerStatus(res.text) };
}

function summarizeCapabilities(xml) {
  const resolutionsOf = (block) =>
    (block.match(/<scan:DiscreteResolution>[\s\S]*?<\/scan:DiscreteResolution>/g) ?? []).map((r) => Number(tag(r, 'scan:XResolution')));
  const platenBlock = xml.match(/<scan:Platen>[\s\S]*?<\/scan:Platen>/)?.[0] ?? '';
  const platenCaps = xml.match(/<scan:PlatenInputCaps>[\s\S]*?<\/scan:PlatenInputCaps>/)?.[0] ?? '';
  const adfBlock = xml.match(/<scan:Adf>[\s\S]*?<\/scan:Adf>/)?.[0] ?? '';
  const adfCaps = xml.match(/<scan:AdfSimplexInputCaps>[\s\S]*?<\/scan:AdfSimplexInputCaps>/)?.[0] ?? '';
  return {
    version: tag(xml, 'pwg:Version'),
    makeAndModel: tag(xml, 'pwg:MakeAndModel'),
    xmlnsScan: xml.match(/xmlns:scan="([^"]+)"/)?.[1],
    xmlnsPwg: xml.match(/xmlns:pwg="([^"]+)"/)?.[1],
    platen: {
      element: '<scan:Platen><scan:PlatenInputCaps>',
      maxWidth: Number(tag(platenCaps, 'scan:MaxWidth')),
      maxHeight: Number(tag(platenCaps, 'scan:MaxHeight')),
      resolutions: resolutionsOf(platenCaps),
      colorModes: tags(platenCaps, 'scan:ColorMode'),
      documentFormats: tags(platenCaps, 'pwg:DocumentFormat'),
      intents: tags(platenBlock, 'scan:Intent'),
      binaryRenderings: tags(platenBlock, 'scan:BinaryRendering'),
    },
    adf: {
      element: '<scan:Adf><scan:AdfSimplexInputCaps>',
      feederCapacity: Number(tag(adfBlock, 'scan:FeederCapacity')),
      adfOptions: tags(adfBlock, 'scan:AdfOption'),
      maxWidth: Number(tag(adfCaps, 'scan:MaxWidth')),
      maxHeight: Number(tag(adfCaps, 'scan:MaxHeight')),
      resolutions: resolutionsOf(adfCaps),
      colorModes: tags(adfCaps, 'scan:ColorMode'),
      documentFormats: tags(adfCaps, 'pwg:DocumentFormat'),
    },
  };
}

function scanSettings({ source, dpi, mode, format, width, height }) {
  return (
    '<scan:ScanSettings xmlns:pwg="http://www.pwg.org/schemas/2010/12/sm" xmlns:scan="http://schemas.hp.com/imaging/escl/2011/05/03">' +
    '<pwg:Version>2.0</pwg:Version>' +
    '<scan:Intent>Document</scan:Intent>' +
    '<pwg:ScanRegions><pwg:ScanRegion>' +
    '<pwg:ContentRegionUnits>escl:ThreeHundredthsOfInches</pwg:ContentRegionUnits>' +
    '<pwg:XOffset>0</pwg:XOffset><pwg:YOffset>0</pwg:YOffset>' +
    `<pwg:Width>${width}</pwg:Width><pwg:Height>${height}</pwg:Height>` +
    '</pwg:ScanRegion></pwg:ScanRegions>' +
    `<pwg:InputSource>${source}</pwg:InputSource>` +
    `<scan:ColorMode>${mode}</scan:ColorMode>` +
    `<pwg:DocumentFormat>${format}</pwg:DocumentFormat>` +
    `<scan:DocumentFormatExt>${format}</scan:DocumentFormatExt>` +
    `<scan:XResolution>${dpi}</scan:XResolution><scan:YResolution>${dpi}</scan:YResolution>` +
    '</scan:ScanSettings>'
  );
}

async function createJob(settingsXml) {
  const started = Date.now();
  const res = await request('/ScanJobs', { method: 'POST', body: settingsXml });
  return {
    status: res.status,
    location: res.headers.location ?? null,
    contentType: res.headers['content-type'] ?? null,
    bodyBytes: res.body.length,
    elapsedMs: Date.now() - started,
  };
}

async function findJob(jobUri) {
  const { parsed } = await getStatus();
  return parsed.jobs.find((j) => j.jobUri === jobUri) ?? null;
}

async function fetchDocuments(jobUri, outDir) {
  const pages = [];
  for (let page = 1; page <= 50; page += 1) {
    const started = Date.now();
    const res = await request(`${stripBase(jobUri)}/NextDocument`, { timeout: DOC_TIMEOUT_MS });
    const elapsedMs = Date.now() - started;
    if (res.status !== 200) {
      pages.push({ page, httpStatus: res.status, contentType: res.headers['content-type'], bytes: res.body.length, elapsedMs, end: true });
      return pages;
    }
    const kind = detectKind(res.body);
    const file = join(outDir, `page-${String(page).padStart(3, '0')}${kind.ext}`);
    await writeFile(file, res.body);
    pages.push({ page, httpStatus: res.status, contentType: res.headers['content-type'], bytes: res.body.length, elapsedMs, kind: kind.name, file: basename(file), meta: kind.meta });
  }
  return pages;
}

function stripBase(uri) {
  return new URL(uri).pathname.replace(/^\/eSCL/, '');
}

function detectKind(buf) {
  if (buf.length >= 3 && buf[0] === 0xff && buf[1] === 0xd8 && buf[2] === 0xff) {
    return { name: 'jpeg', ext: '.jpg', meta: jpegSize(buf) };
  }
  if (buf.subarray(0, 5).toString('latin1') === '%PDF-') {
    return { name: 'pdf', ext: '.pdf', meta: { version: buf.subarray(0, 8).toString('latin1') } };
  }
  return { name: 'unknown', ext: '.bin', meta: { magic: buf.subarray(0, 8).toString('hex') } };
}

function jpegSize(buf) {
  let i = 2;
  while (i < buf.length - 9) {
    if (buf[i] !== 0xff) { i += 1; continue; }
    const marker = buf[i + 1];
    const len = buf.readUInt16BE(i + 2);
    if (marker >= 0xc0 && marker <= 0xcf && ![0xc4, 0xc8, 0xcc].includes(marker)) {
      return { height: buf.readUInt16BE(i + 5), width: buf.readUInt16BE(i + 7), components: buf[i + 9], sofMarker: `0x${marker.toString(16)}` };
    }
    i += 2 + len;
  }
  return { error: 'SOF not found' };
}

async function deleteJob(jobUri) {
  const res = await request(stripBase(jobUri), { method: 'DELETE' });
  return { status: res.status, bodyBytes: res.body.length };
}

async function cmdCaps() {
  console.log(JSON.stringify(summarizeCapabilities(await getCapabilities()), null, 2));
}

async function cmdStatus() {
  const { parsed } = await getStatus();
  console.log(JSON.stringify(parsed, null, 2));
}

async function cmdScan(opts) {
  const caps = summarizeCapabilities(await getCapabilities());
  const srcCaps = opts.source === 'Platen' ? caps.platen : caps.adf;
  const width = opts.width ?? srcCaps.maxWidth;
  const height = opts.height ?? srcCaps.maxHeight;
  const settings = scanSettings({ ...opts, width, height });
  await mkdir(opts.out, { recursive: true });
  await writeFile(join(opts.out, 'ScanSettings.xml'), settings);

  let job;
  for (let attempt = 0; ; attempt += 1) {
    job = await createJob(settings);
    if (job.status === 201 && job.location) break;
    if (job.status !== 503 || attempt >= 2) {
      throw new Error(`ScanJobs rejected: HTTP ${job.status} (expected 201 + Location)`);
    }
    await new Promise((r) => setTimeout(r, 3000));
  }

  const jobStatusProbe = await request(`${stripBase(job.location)}/ScannerJobStatus`).then(
    (r) => ({ httpStatus: r.status, contentType: r.headers['content-type'] }),
    (e) => ({ error: String(e.message) }),
  );

  try {
    const pages = await fetchDocuments(job.location, opts.out);
    const final = await findJob(new URL(job.location).pathname);
    const after = await getStatus();
    console.log(JSON.stringify({ job, jobStatusProbe, requested: opts, final, pages, statusAfter: after.parsed }, null, 2));
  } finally {
    const deleted = await deleteJob(job.location).catch((e) => ({ error: String(e.message) }));
    const afterDelete = (await getStatus().catch(() => ({ parsed: null }))).parsed;
    console.error(`# delete=${JSON.stringify(deleted)} deviceState=${afterDelete?.state}`);
  }
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    source: { type: 'string', default: 'Platen' },
    width: { type: 'string' },
    height: { type: 'string' },
    dpi: { type: 'string', default: '300' },
    mode: { type: 'string', default: 'Grayscale8' },
    format: { type: 'string', default: 'image/jpeg' },
    out: { type: 'string', default: './scans' },
  },
});

const command = positionals[0] ?? 'caps';
const SOURCES = { platen: 'Platen', feeder: 'Feeder', adf: 'Feeder' };
const sourceArg = SOURCES[values.source.toLowerCase()];
if (!sourceArg) throw new Error(`unknown source: ${values.source} (use Platen|Feeder)`);

const scanOpts = {
  source: sourceArg,
  dpi: Number(values.dpi),
  mode: values.mode,
  format: values.format,
  out: values.out,
  width: values.width === undefined ? undefined : Number(values.width),
  height: values.height === undefined ? undefined : Number(values.height),
};

try {
  if (command === 'caps') await cmdCaps();
  else if (command === 'status') await cmdStatus();
  else if (command === 'scan') await cmdScan(scanOpts);
  else throw new Error(`unknown command: ${command}`);
} catch (err) {
  console.error(`escl-probe: ${err.message}`);
  process.exitCode = 1;
}
