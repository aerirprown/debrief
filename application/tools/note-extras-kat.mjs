/**
 * Game note extras checks for Debrief (Plan step 9).
 * Reminders that fire, voice memo, second revision / fork rules.
 *
 *   node --test tools/note-extras-kat.mjs
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const journalCrate = path.join(__dirname, '..', 'crates', 'debrief-journal');
const web = path.join(__dirname, '..', 'packages', 'web', 'src');

test('note extras plan check: reminder, voice, fork', () => {
  const result = spawnSync(
    'cargo',
    ['test', '--test', 'plan_check', 'note_extras_reminder_voice_and_fork', '--', '--exact', '--nocapture'],
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
  assert.match(result.stdout + result.stderr, /note_extras_reminder_voice_and_fork/);
});

test('freeform page model supports audio + reminders', () => {
  const pageTs = fs.readFileSync(path.join(web, 'freeform', 'page.ts'), 'utf8');
  assert.match(pageTs, /kind: 'audio'/);
  assert.match(pageTs, /FreeformReminder/);
  assert.match(pageTs, /export function addAudio/);
  assert.match(pageTs, /export function addReminder/);
  assert.match(pageTs, /export function clearReminder/);
});

test('mid-session prompt + note extras API exist', () => {
  const prompt = fs.readFileSync(path.join(web, 'games', 'MidSessionPrompt.tsx'), 'utf8');
  assert.match(prompt, /debrief-mid-session/);
  assert.match(prompt, /mid_session_submit/);
  const api = fs.readFileSync(path.join(web, 'games', 'noteExtrasApi.ts'), 'utf8');
  assert.match(api, /note_due_notifications/);
  assert.match(api, /fireDueJournalNotifications/);
  const canvas = fs.readFileSync(path.join(web, 'freeform', 'FreeformCanvas.tsx'), 'utf8');
  assert.match(canvas, /freeform-voice/);
  assert.match(canvas, /freeform-remind-add/);
});
