/**
 * Two PCs + cloud folder checks for Debrief (Plan step 10).
 *
 *   node --test tools/cloud-journal-kat.mjs
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const journalCrate = path.join(__dirname, '..', 'crates', 'debrief-journal');
const repoRoot = path.join(__dirname, '..', '..');
const web = path.join(__dirname, '..', 'packages', 'web', 'src');

test('cloud folder plan check: two devices, listing + memoir + fork', () => {
  const result = spawnSync(
    'cargo',
    ['test', '--test', 'plan_check', 'two_pcs_cloud_folder_sessions_and_forks', '--', '--exact', '--nocapture'],
    {
      cwd: journalCrate,
      encoding: 'utf8',
      shell: true,
      timeout: 120_000,
    },
  );
  if (result.status !== 0) {
    console.error(result.stdout);
    console.error(result.stderr);
  }
  assert.equal(result.status, 0);
  assert.match(result.stdout + result.stderr, /two_pcs_cloud_folder_sessions_and_forks/);
});

test('cloud folder how-to doc exists', () => {
  const doc = path.join(repoRoot, 'docs', 'gaming-journal-cloud.md');
  assert.ok(fs.existsSync(doc), `missing ${doc}`);
  const text = fs.readFileSync(doc, 'utf8');
  assert.match(text, /Dropbox|OneDrive|Syncthing/);
  assert.match(text, /device id/i);
  assert.match(text, /\.git/);
  assert.match(text, /sessions\//);
});

test('Games shelf exposes journal folder picker + cloud help', () => {
  const shelf = fs.readFileSync(path.join(web, 'games', 'GamesShelf.tsx'), 'utf8');
  assert.match(shelf, /Journal folder/);
  assert.match(shelf, /journal_pick_folder|pickJournalFolder/);
  assert.match(shelf, /cloud|Dropbox|sync client/i);
});
