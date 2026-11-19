/**
 * Watcher / quit-prompt checks for Debrief (Plan step 7).
 * The authoritative check is the Rust probe test; this KAT documents the
 * expected session-finish shapes the prompt writes.
 *
 *   node --test tools/watcher-kat.mjs
 *   cargo test --manifest-path crates/debrief-watcher/Cargo.toml
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const crate = path.join(__dirname, '..', 'crates', 'debrief-watcher');

test('watcher crate plan checks (probe process → prompt → file)', () => {
  const result = spawnSync(
    'cargo',
    ['test', '--test', 'plan_check', '--', '--nocapture'],
    {
      cwd: crate,
      encoding: 'utf8',
      shell: true,
      timeout: 120_000,
    },
  );
  if (result.status !== 0) {
    console.error(result.stdout);
    console.error(result.stderr);
  }
  assert.equal(result.status, 0, 'cargo test plan_check should pass');
  assert.match(result.stdout + result.stderr, /real_probe_process_start_stop_prompt_and_file/);
  assert.match(result.stdout + result.stderr, /mock_short_lived_process_prompts_and_writes_session/);
});

test('quit prompt UI module exists', () => {
  const prompt = path.join(
    __dirname,
    '..',
    'packages',
    'web',
    'src',
    'games',
    'QuitPrompt.tsx',
  );
  assert.ok(fs.existsSync(prompt));
  const src = fs.readFileSync(prompt, 'utf8');
  assert.match(src, /watcher_submit/);
  assert.match(src, /watcher_dismiss/);
  assert.match(src, /Escape/);
});
