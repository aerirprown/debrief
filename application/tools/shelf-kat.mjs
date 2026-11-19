/**
 * Games shelf checks for Debrief (Plan step 8).
 * One Steam install (fixture) + one manual game on the same shelf.
 *
 *   node --test tools/shelf-kat.mjs
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const steamFixture = path.join(__dirname, 'fixtures', 'steam');
const journalCrate = path.join(__dirname, '..', 'crates', 'debrief-journal');

test('steam fixture has Fixture Quest manifest', () => {
  const acf = path.join(steamFixture, 'steamapps', 'appmanifest_424242.acf');
  assert.ok(fs.existsSync(acf));
  const text = fs.readFileSync(acf, 'utf8');
  assert.match(text, /"appid"\s+"424242"/);
  assert.match(text, /"name"\s+"Fixture Quest"/);
});

test('shelf plan check: steam + manual in one shelf', () => {
  const result = spawnSync(
    'cargo',
    ['test', '--test', 'plan_check', 'shelf_has_steam_install_and_manual_game', '--', '--exact', '--nocapture'],
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
  assert.match(result.stdout + result.stderr, /shelf_has_steam_install_and_manual_game/);
});

test('GamesShelf UI module exists', () => {
  const shelf = path.join(__dirname, '..', 'packages', 'web', 'src', 'games', 'GamesShelf.tsx');
  assert.ok(fs.existsSync(shelf));
  const src = fs.readFileSync(shelf, 'utf8');
  assert.match(src, /debrief-games-shelf/);
  assert.match(src, /shelf-launch/);
  assert.match(src, /Add game/);
});
