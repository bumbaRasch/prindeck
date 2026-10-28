#!/usr/bin/env node

import dgram from 'node:dgram';
import { parseArgs } from 'node:util';

function tlv(type, content) {
  if (content.length < 0x80) return Buffer.concat([Buffer.from([type, content.length]), content]);
  return Buffer.concat([Buffer.from([type, 0x81, content.length]), content]);
}

function encodeInteger(value) {
  const bytes = [];
  let n = value;
  do {
    bytes.unshift(n & 0xff);
    n >>= 8;
  } while (n !== 0 && n !== -1);
  if (bytes[0] & 0x80) bytes.unshift(value < 0 ? 0xff : 0x00);
  return tlv(0x02, Buffer.from(bytes));
}

function encodeOid(oid) {
  const arcs = oid.split('.').map(Number);
  const out = [40 * arcs[0] + arcs[1]];
  for (const arc of arcs.slice(2)) {
    const chunk = [arc & 0x7f];
    let rest = Math.floor(arc / 128);
    while (rest > 0) {
      chunk.unshift((rest & 0x7f) | 0x80);
      rest = Math.floor(rest / 128);
    }
    out.push(...chunk);
  }
  return tlv(0x06, Buffer.from(out));
}

function encodeOctetString(value) {
  const b = Buffer.isBuffer(value) ? value : Buffer.from(value, 'latin1');
  return tlv(0x04, b);
}

const NULL_VALUE = tlv(0x05, Buffer.alloc(0));

function buildGetRequest(community, requestId, oids) {
  const varbinds = oids.map((oid) => tlv(0x30, Buffer.concat([encodeOid(oid), NULL_VALUE])));
  const pduBody = Buffer.concat([
    encodeInteger(requestId),
    encodeInteger(0),
    encodeInteger(0),
    tlv(0x30, Buffer.concat(varbinds)),
  ]);
  const message = Buffer.concat([
    encodeInteger(1),
    encodeOctetString(community),
    tlv(0xa0, pduBody),
  ]);
  return tlv(0x30, message);
}

function readTlv(buf, offset) {
  const type = buf[offset];
  let len = buf[offset + 1];
  let header = 2;
  if (len & 0x80) {
    const count = len & 0x7f;
    len = 0;
    for (let i = 0; i < count; i += 1) len = (len << 8) | buf[offset + 2 + i];
    header += count;
  }
  const start = offset + header;
  return { type, length: len, start, end: start + len, next: start + len };
}

function decodeValue(buf, node) {
  const raw = buf.subarray(node.start, node.end);
  switch (node.type) {
    case 0x02: {
      let v = raw[0] & 0x80 ? raw[0] - 0x100 : raw[0];
      for (let i = 1; i < raw.length; i += 1) v = v * 256 + raw[i];
      return { kind: 'integer', value: v };
    }
    case 0x04:
      return { kind: 'octet-string', value: raw.toString('latin1'), hex: raw.toString('hex'), bytes: raw.length };
    case 0x05:
      return { kind: 'null', value: null };
    case 0x06:
      return { kind: 'oid', value: decodeOid(raw) };
    case 0x41:
      return { kind: 'counter32', value: raw.length === 0 ? null : raw.reduce((a, b) => a * 256 + b, 0), hex: raw.toString('hex'), bytes: raw.length };
    case 0x42:
      return { kind: 'gauge32', value: raw.length === 0 ? null : raw.reduce((a, b) => a * 256 + b, 0), hex: raw.toString('hex'), bytes: raw.length };
    case 0x43:
      return { kind: 'timeticks', value: raw.length === 0 ? null : raw.reduce((a, b) => a * 256 + b, 0), hex: raw.toString('hex'), bytes: raw.length };
    case 0x44:
      return { kind: 'opaque', value: raw.toString('hex'), hex: raw.toString('hex'), bytes: raw.length };
    case 0x46: {
      let v = 0n;
      for (const b of raw) v = (v << 8n) | BigInt(b);
      return { kind: 'counter64', value: v.toString() };
    }
    case 0x80:
      return { kind: 'noSuchObject', value: null };
    case 0x81:
      return { kind: 'noSuchInstance', value: null };
    case 0x82:
      return { kind: 'endOfMibView', value: null };
    default:
      return { kind: `unknown-0x${node.type.toString(16)}`, value: raw.toString('hex') };
  }
}

function decodeOid(bytes) {
  const arcs = [Math.floor(bytes[0] / 40), bytes[0] % 40];
  let value = 0;
  for (let i = 1; i < bytes.length; i += 1) {
    value = value * 128 + (bytes[i] & 0x7f);
    if ((bytes[i] & 0x80) === 0) {
      arcs.push(value);
      value = 0;
    }
  }
  return arcs.join('.');
}

function decodeResponse(buf) {
  const message = readTlv(buf, 0);
  let offset = message.start;
  const version = readTlv(buf, offset);
  offset = version.next;
  const community = readTlv(buf, offset);
  offset = community.next;
  const pdu = readTlv(buf, offset);
  if (pdu.type !== 0xa2) throw new Error(`unexpected PDU type 0x${pdu.type.toString(16)} (expected GetResponse)`);

  let cursor = pdu.start;
  const requestIdNode = readTlv(buf, cursor);
  cursor = requestIdNode.next;
  const statusNode = readTlv(buf, cursor);
  cursor = statusNode.next;
  const indexNode = readTlv(buf, cursor);
  cursor = indexNode.next;
  const varbindList = readTlv(buf, cursor);

  const varbinds = [];
  let vb = varbindList.start;
  while (vb < varbindList.end) {
    const entry = readTlv(buf, vb);
    let inner = entry.start;
    const oidNode = readTlv(buf, inner);
    inner = oidNode.next;
    const valueNode = readTlv(buf, inner);
    varbinds.push({
      oid: decodeOid(buf.subarray(oidNode.start, oidNode.end)),
      ...decodeValue(buf, valueNode),
    });
    vb = entry.next;
  }

  return {
    version: buf.subarray(version.start, version.end)[0],
    community: buf.subarray(community.start, community.end).toString('latin1'),
    requestId: buf.subarray(requestIdNode.start, requestIdNode.end).reduce((a, b) => a * 256 + b, 0),
    errorStatus: buf.subarray(statusNode.start, statusNode.end).reduce((a, b) => a * 256 + b, 0),
    errorIndex: buf.subarray(indexNode.start, indexNode.end).reduce((a, b) => a * 256 + b, 0),
    varbinds,
  };
}

let nextRequestId = 1;

function getOnce({ host, port, community, timeout, oids }) {
  return new Promise((resolve, reject) => {
    const socket = dgram.createSocket('udp4');
    const requestId = nextRequestId++;
    const packet = buildGetRequest(community, requestId, oids);
    const timer = setTimeout(() => {
      socket.close();
      reject(new Error(`timeout after ${timeout} ms`));
    }, timeout);

    socket.once('error', (err) => {
      clearTimeout(timer);
      socket.close();
      reject(err);
    });

    socket.once('message', (msg) => {
      clearTimeout(timer);
      socket.close();
      try {
        resolve({ response: decodeResponse(msg) });
      } catch (err) {
        reject(err);
      }
    });

    socket.send(packet, port, host, (err) => {
      if (err) {
        clearTimeout(timer);
        socket.close();
        reject(err);
      }
    });
  });
}

async function get(opts) {
  let lastError;
  for (let attempt = 1; attempt <= opts.retries; attempt += 1) {
    const started = Date.now();
    try {
      const { response } = await getOnce({ ...opts, oids: [opts.oid] });
      const varbind = response.varbinds[0];
      if (response.errorStatus !== 0) {
        throw new Error(`agent error-status=${response.errorStatus} index=${response.errorIndex}`);
      }
      return { ...varbind, elapsedMs: Date.now() - started, attempt };
    } catch (err) {
      lastError = err;
    }
  }
  throw new Error(`${opts.oid}: ${lastError.message} (${opts.retries} attempts)`);
}

async function getBatch(opts) {
  let lastError;
  for (let attempt = 1; attempt <= 2; attempt += 1) {
    const started = Date.now();
    try {
      const { response } = await getOnce({ ...opts, oids: BATCH_OIDS.map(([oid]) => oid) });
      if (response.errorStatus !== 0) {
        throw new Error(`agent error-status=${response.errorStatus} index=${response.errorIndex}`);
      }
      return { varbinds: response.varbinds, elapsedMs: Date.now() - started, attempt };
    } catch (err) {
      lastError = err;
    }
  }
  throw new Error(`batched get of ${BATCH_OIDS.length} OIDs: ${lastError.message} (2 attempts)`);
}

const OIDS = [
  ['1.3.6.1.2.1.43.11.1.1.6.1.1', 'prtMarkerSuppliesDescription.1.1'],
  ['1.3.6.1.2.1.43.11.1.1.6.1.2', 'prtMarkerSuppliesDescription.1.2'],
  ['1.3.6.1.2.1.43.11.1.1.6.1.3', 'prtMarkerSuppliesDescription.1.3'],
  ['1.3.6.1.2.1.43.11.1.1.8.1.1', 'prtMarkerSuppliesMaxCapacity.1.1'],
  ['1.3.6.1.2.1.43.11.1.1.8.1.2', 'prtMarkerSuppliesMaxCapacity.1.2'],
  ['1.3.6.1.2.1.43.11.1.1.9.1.1', 'prtMarkerSuppliesLevel.1.1'],
  ['1.3.6.1.2.1.43.11.1.1.9.1.2', 'prtMarkerSuppliesLevel.1.2'],
  ['1.3.6.1.2.1.43.11.1.1.9.1.3', 'prtMarkerSuppliesLevel.1.3'],
  ['1.3.6.1.2.1.43.10.2.1.4.1.1', 'prtMarkerLifeCount.1.1'],
  ['1.3.6.1.2.1.25.3.2.1.5.1', 'hrDeviceStatus.1'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.8.0', 'brInfoMaintenance'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.10.0', 'brInfoCounter'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.11.0', 'brInfoNextCare'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.18.0', 'brInfoCoverage'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.20.0', 'brInfoReplaceCount'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.21.0', 'brInfoJamCount'],
];

const BATCH_OIDS = [
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.8.0', 'brInfoMaintenance'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.10.0', 'brInfoCounter'],
  ['1.3.6.1.4.1.2435.2.3.9.4.2.1.5.5.11.0', 'brInfoNextCare'],
];

function decodeRecords(hex) {
  const bytes = Buffer.from(hex, 'hex');
  const records = [];
  let offset = 0;
  while (offset < bytes.length && bytes[offset] !== 0xff) {
    if (offset + 7 > bytes.length) throw new Error('record is truncated');
    const id = bytes[offset];
    if (bytes[offset + 1] !== 1 || bytes[offset + 2] !== 4) {
      throw new Error(`record 0x${id.toString(16)} does not match the <id> 01 04 <uint32> shape`);
    }
    records.push([id, bytes.readUInt32BE(offset + 3)]);
    offset += 7;
  }
  return records;
}

function recordValue(records, id) {
  const found = records.find(([key]) => key === id);
  return found === undefined ? null : found[1];
}

function decodeCounters(varbinds) {
  const [maintenance, counter, nextCare] = varbinds.map((vb) => decodeRecords(vb.hex));
  const drumHundredths = recordValue(maintenance, 0x41);
  return {
    tonerPercent: recordValue(maintenance, 0x81),
    drumPercent: drumHundredths === null ? null : Math.floor(drumHundredths / 100),
    pagesTotal: recordValue(maintenance, 0x11),
    duplexPages: recordValue(counter, 0x06),
    drumPagesLeft: recordValue(nextCare, 0x82),
  };
}

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    host: { type: 'string', default: '192.0.2.10' },
    port: { type: 'string', default: '161' },
    community: { type: 'string', default: 'public' },
    timeout: { type: 'string', default: '2000' },
    retries: { type: 'string', default: '3' },
    json: { type: 'boolean', default: false },
  },
});

const opts = {
  host: values.host,
  port: Number(values.port),
  community: values.community,
  timeout: Number(values.timeout),
  retries: Number(values.retries),
};

const queries = positionals.length > 0 ? positionals.map((oid) => [oid, oid]) : OIDS;

const results = [];
let failures = 0;

for (const [oid, label] of queries) {
  const timestamp = new Date().toISOString();
  try {
    const r = await get({ ...opts, oid });
    results.push({
      timestamp,
      label,
      oid: r.oid,
      kind: r.kind,
      value: r.value,
      hex: r.hex,
      bytes: r.bytes,
      elapsedMs: r.elapsedMs,
      attempt: r.attempt,
    });
  } catch (err) {
    failures += 1;
    results.push({ timestamp, label, oid, error: err.message });
  }
}

const batchTimestamp = new Date().toISOString();
let batch;
try {
  const r = await getBatch(opts);
  batch = {
    timestamp: batchTimestamp,
    varbinds: r.varbinds.map((vb) => ({ ...vb, label: BATCH_OIDS.find(([oid]) => oid === vb.oid)?.[1] ?? vb.oid })),
    elapsedMs: r.elapsedMs,
    attempt: r.attempt,
    counters: decodeCounters(r.varbinds),
  };
} catch (err) {
  failures += 1;
  batch = { timestamp: batchTimestamp, error: err.message };
}

if (values.json) {
  console.log(JSON.stringify({ community: opts.community, timeoutMs: opts.timeout, retries: opts.retries, results, batch }, null, 2));
} else {
  for (const r of results) {
    if (r.error) {
      console.log(`${r.label.padEnd(36)} ${r.oid.padEnd(44)} ERROR ${r.error}`);
    } else {
      const printable = r.kind === 'octet-string' ? JSON.stringify(r.value) : JSON.stringify(r.value);
      const hex = r.hex ? ` hex=${r.hex}` : '';
      console.log(`${r.label.padEnd(36)} ${r.oid.padEnd(44)} ${r.kind.padEnd(12)} ${printable}${hex} (${r.elapsedMs} ms)`);
    }
  }
  console.log('batched GET (3 varbinds, one PDU — the shape the Rust client uses)');
  if (batch.error) {
    console.log(`  ERROR ${batch.error}`);
  } else {
    for (const vb of batch.varbinds) {
      console.log(`  ${vb.label.padEnd(36)} ${vb.oid.padEnd(44)} ${vb.kind.padEnd(12)} ${vb.bytes} B hex=${vb.hex} (${batch.elapsedMs} ms)`);
    }
    console.log(`  decoded ${JSON.stringify(batch.counters)}`);
  }
}

if (results.every((r) => r.error)) {
  console.error('probe-snmp: no OID answered');
  process.exitCode = 1;
} else if (failures > 0) {
  console.error(`probe-snmp: ${failures}/${results.length} OIDs failed`);
  process.exitCode = 1;
}
