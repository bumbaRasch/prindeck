#!/usr/bin/env node

import { execFileSync } from 'node:child_process';

const REPO = 'bumbaRasch/prindeck';
const LABELS_ENDPOINT = `/repos/${REPO}/labels`;

const TAXONOMY = [
  { name: 'type: bug', color: 'd73a4a', description: 'Something does not work as documented or expected' },
  { name: 'type: feature', color: 'a2eeef', description: 'New capability or behaviour request' },
  { name: 'area: scan', color: '1d76db', description: 'Scan screen and the eSCL pipeline' },
  { name: 'area: overview', color: '1d76db', description: 'Overview screen and SNMP status data' },
  { name: 'area: print', color: '1d76db', description: 'Print screen and the CUPS pipeline' },
  { name: 'area: settings', color: '1d76db', description: 'Settings screen and config handling' },
  { name: 'area: tray', color: '1d76db', description: 'Tray icon, menu, autostart, window behaviour' },
  { name: 'os: wayland', color: 'c2e0c6', description: 'Reproduces on a Wayland session' },
  { name: 'os: x11', color: 'c2e0c6', description: 'Reproduces on an X11 session' },
  { name: 'status: needs-triage', color: 'eade9b', description: 'Fresh report, not yet reviewed by the maintainer' },
  { name: 'status: confirmed', color: '0e8a16', description: 'Reproduced and accepted for fixing' },
  { name: 'status: duplicate', color: 'd4c5f9', description: 'Same root cause as an existing issue' },
  { name: 'status: wontfix', color: 'f9d0c4', description: 'Deliberately not addressed (see PRD non-goals)' },
  { name: 'priority: high', color: 'd93f0b', description: 'Breaks a core flow: scanning or printing' },
  { name: 'priority: low', color: 'fef2c0', description: 'Cosmetic or minor inconvenience' },
];

const DELETE = ['accessibility', 'bug', 'documentation', 'duplicate', 'enhancement', 'good first issue', 'help wanted', 'invalid', 'javascript', 'question', 'wontfix'];

const args = process.argv.slice(2);
const unknown = args.filter((arg) => arg !== '--dry-run');
if (unknown.length > 0) {
  console.error(`usage: node scripts/setup-labels.mjs [--dry-run] (unknown: ${unknown.join(', ')})`);
  process.exit(1);
}
const dryRun = args.includes('--dry-run');

const labelEndpoint = (name) => `${LABELS_ENDPOINT}/${encodeURIComponent(name)}`;

function runGh(arguments_, parse) {
  const output = execFileSync('gh', ['api', ...arguments_], { encoding: 'utf8' });
  return parse ? JSON.parse(output) : null;
}

function fetchLabels() {
  return runGh([LABELS_ENDPOINT, '--paginate', '--slurp'], true).flat();
}

function createLabel(label) {
  runGh([
    LABELS_ENDPOINT,
    '--method', 'POST',
    '--raw-field', `name=${label.name}`,
    '--raw-field', `color=${label.color}`,
    '--raw-field', `description=${label.description}`,
  ]);
}

function updateLabel(label) {
  runGh([
    labelEndpoint(label.name),
    '--method', 'PATCH',
    '--raw-field', `color=${label.color}`,
    '--raw-field', `description=${label.description}`,
  ]);
}

function deleteLabel(name) {
  try {
    runGh([labelEndpoint(name), '--method', 'DELETE']);
    return 'deleted';
  } catch (err) {
    if (String(err.stderr ?? '').includes('HTTP 404')) return 'already absent';
    throw err;
  }
}

try {
  const existing = new Map(fetchLabels().map((label) => [label.name, label]));
  const creates = TAXONOMY.filter((label) => !existing.has(label.name));
  const updates = TAXONOMY.filter((label) => {
    const current = existing.get(label.name);
    return current !== undefined && (current.color !== label.color || current.description !== label.description);
  });

  console.log(dryRun ? 'dry run: no changes will be applied' : `applying label taxonomy to ${REPO}`);
  for (const label of creates) {
    if (dryRun) console.log(`would create: ${label.name} #${label.color} "${label.description}"`);
    else {
      createLabel(label);
      console.log(`created: ${label.name}`);
    }
  }
  for (const label of updates) {
    if (dryRun) console.log(`would update: ${label.name} #${label.color} "${label.description}"`);
    else {
      updateLabel(label);
      console.log(`updated: ${label.name}`);
    }
  }
  for (const name of DELETE) {
    if (dryRun) console.log(`would delete: ${name}`);
    else console.log(`${deleteLabel(name)}: ${name}`);
  }
  console.log(`${dryRun ? 'plan' : 'done'}: ${creates.length + updates.length} create/update, ${DELETE.length} delete`);
} catch (err) {
  console.error(`setup-labels: ${err.message}`);
  process.exitCode = 1;
}
