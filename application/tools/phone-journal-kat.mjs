/**
 * Android / iOS journal path checks for Debrief (Plan step 11).
 *
 *   node --test tools/phone-journal-kat.mjs
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
const fixture = path.join(__dirname, 'fixtures', 'journal-phone');
const tauriSrc = path.join(__dirname, '..', 'packages', 'desktop', 'src-tauri');

test('phone fixture plan check: shelf, return card, reminders, no watcher', () => {
  const result = spawnSync(
    'cargo',
    ['test', '--test', 'plan_check', 'phone_fixture_journal_without_watcher', '--', '--exact', '--nocapture'],
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
  assert.match(result.stdout + result.stderr, /phone_fixture_journal_without_watcher/);
});

test('phone fixture folder has game, session, note reminder', () => {
  const game = path.join(fixture, 'games', 'game-phone-1');
  const sessions = path.join(fixture, 'sessions');
  const notes = path.join(fixture, 'notes', 'note-phone-1');
  assert.ok(fs.existsSync(game), `missing ${game}`);
  assert.ok(fs.existsSync(sessions), `missing ${sessions}`);
  assert.ok(fs.existsSync(notes), `missing ${notes}`);
  const sessionFiles = fs.readdirSync(sessions).filter((n) => n.endsWith('.json'));
  assert.ok(sessionFiles.length >= 1);
  const session = JSON.parse(fs.readFileSync(path.join(sessions, sessionFiles[0]), 'utf8'));
  assert.equal(session.text, 'logged from phone');
  assert.equal(session.game_id, 'game-phone-1');
  const noteFiles = fs.readdirSync(notes).filter((n) => n.endsWith('.json'));
  const note = JSON.parse(fs.readFileSync(path.join(notes, noteFiles[0]), 'utf8'));
  assert.ok(Array.isArray(note.reminders) && note.reminders.length >= 1);
  assert.ok(note.containers.some((c) => c.kind === 'audio'));
});

test('phone how-to doc exists', () => {
  const doc = path.join(repoRoot, 'docs', 'gaming-journal-phone.md');
  assert.ok(fs.existsSync(doc), `missing ${doc}`);
  const text = fs.readFileSync(doc, 'utf8');
  assert.match(text, /Android|iOS/);
  assert.match(text, /watcher/i);
  assert.match(text, /Journal folder/);
  assert.match(text, /Log a session|manual/i);
});

test('Games shelf phone mode: log session, no Play/Steam, schedule API', () => {
  const shelf = fs.readFileSync(path.join(web, 'games', 'GamesShelf.tsx'), 'utf8');
  assert.match(shelf, /data-phone/);
  assert.match(shelf, /Log a session|shelf-log-session/);
  assert.match(shelf, /addManualSession|journal_add_session/);
  assert.match(shelf, /phone/);
  const api = fs.readFileSync(path.join(web, 'games', 'shelfApi.ts'), 'utf8');
  assert.match(api, /journal_add_session/);
  assert.match(api, /phone/);
  const extras = fs.readFileSync(path.join(web, 'games', 'noteExtrasApi.ts'), 'utf8');
  assert.match(extras, /scheduleJournalNotifications/);
  const canvas = fs.readFileSync(path.join(web, 'freeform', 'FreeformCanvas.tsx'), 'utf8');
  assert.match(canvas, /pinch|touchstart|setZoom/);
});

test('mobile Tauri path: journal without watcher; android + apple gen present', () => {
  const lib = fs.readFileSync(path.join(tauriSrc, 'src', 'lib.rs'), 'utf8');
  assert.match(lib, /#\[cfg\(mobile\)\]/);
  assert.match(lib, /journal_add_session/);
  // Watcher only on win/linux desktop.
  assert.match(lib, /cfg\(any\(target_os = "windows", target_os = "linux"\)\)/);
  assert.ok(fs.existsSync(path.join(tauriSrc, 'gen', 'android', 'app')), 'android gen missing');
  assert.ok(fs.existsSync(path.join(tauriSrc, 'gen', 'apple')), 'apple/iOS gen missing');
  const cargo = fs.readFileSync(path.join(tauriSrc, 'Cargo.toml'), 'utf8');
  assert.match(cargo, /debrief-journal/);
  assert.match(cargo, /target_os = "windows".*debrief-watcher|debrief-watcher[\s\S]*windows/s);
});
