/**
 * Games UI / return card checks for Debrief (Plan step 6).
 *
 * Loads the fixture journal folder and derives a return card:
 * one line, one frame, and a day count.
 *
 *   node --test tools/games-kat.mjs
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const FIXTURE = path.join(__dirname, 'fixtures', 'journal');
const AS_OF = '2026-11-18';

function dayNumber(ymd) {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(ymd);
  if (!m) return null;
  let y = Number(m[1]);
  const month = Number(m[2]);
  const d = Number(m[3]);
  if (month <= 2) y -= 1;
  const era = Math.floor(y / 400);
  const yoe = y - era * 400;
  const mpb = month > 2 ? month - 3 : month + 9;
  const doy = Math.floor((153 * mpb + 2) / 5) + d - 1;
  const doe = yoe * 365 + Math.floor(yoe / 4) - Math.floor(yoe / 100) + doy;
  return era * 146097 + doe - 719468;
}

function daysSinceStopped(stoppedAt, asOfDay) {
  const t = stoppedAt.indexOf('T');
  const stopDay = t >= 0 ? stoppedAt.slice(0, t) : stoppedAt.slice(0, 10);
  const stop = dayNumber(stopDay);
  const asOf = dayNumber(asOfDay);
  if (stop == null || asOf == null || asOf <= stop) return 0;
  return asOf - stop;
}

function readJsonFiles(dir) {
  if (!fs.existsSync(dir)) return [];
  const out = [];
  for (const name of fs.readdirSync(dir)) {
    const full = path.join(dir, name);
    const st = fs.statSync(full);
    if (st.isDirectory()) {
      out.push(...readJsonFiles(full));
      continue;
    }
    if (!name.endsWith('.json')) continue;
    out.push(JSON.parse(fs.readFileSync(full, 'utf8')));
  }
  return out;
}

function loadFixtureJournal(root) {
  const games = readJsonFiles(path.join(root, 'games'));
  const sessions = readJsonFiles(path.join(root, 'sessions'));
  return { games, sessions };
}

function newestSession(sessions, gameId) {
  let best = null;
  for (const s of sessions) {
    if (s.game_id !== gameId) continue;
    if (
      !best
      || s.stopped_at > best.stopped_at
      || (s.stopped_at === best.stopped_at && s.started_at > best.started_at)
      || (s.stopped_at === best.stopped_at && s.started_at === best.started_at && s.id > best.id)
    ) {
      best = s;
    }
  }
  return best;
}

function returnCardFor(game, sessions, asOfDay) {
  const session = newestSession(sessions, game.id);
  const frameId = session?.frame_id ?? null;
  return {
    gameId: game.id,
    title: game.title,
    line: session?.text ?? '',
    whatNext: session?.what_next ?? '',
    daysSince: session ? daysSinceStopped(session.stopped_at, asOfDay) : 0,
    frameId,
    framePath: frameId ? path.join('frames', `${frameId}.png`) : null,
  };
}

function emptyGamePageBody(gameId) {
  return JSON.stringify({ type: 'debrief-game', version: 1, gameId });
}

test('fixture journal: return card has one line, one frame, day count', () => {
  const { games, sessions } = loadFixtureJournal(FIXTURE);
  assert.equal(games.length, 1);
  assert.equal(sessions.length, 1);

  const card = returnCardFor(games[0], sessions, AS_OF);
  assert.equal(card.line, 'cleared the chapel');
  assert.equal(card.whatNext, 'open the side door');
  assert.equal(card.frameId, 'frame-fixture-1');
  assert.equal(card.daysSince, 7);

  const frameFile = path.join(FIXTURE, card.framePath);
  assert.ok(fs.existsSync(frameFile), `missing frame file: ${frameFile}`);
  const bytes = fs.readFileSync(frameFile);
  assert.ok(bytes.length > 8);
  assert.equal(bytes[0], 0x89);
  assert.equal(bytes[1], 0x50); // P
  assert.equal(bytes[2], 0x4e); // N
  assert.equal(bytes[3], 0x47); // G
});

test('fixture journal: list games for folder placement', () => {
  const { games } = loadFixtureJournal(FIXTURE);
  assert.equal(games[0].title, 'Fixture Souls');
  assert.equal(games[0].id, 'game-fixture-1');
  const body = emptyGamePageBody(games[0].id);
  const parsed = JSON.parse(body);
  assert.equal(parsed.type, 'debrief-game');
  assert.equal(parsed.gameId, 'game-fixture-1');
});

test('days since: same day is zero', () => {
  assert.equal(daysSinceStopped('2026-11-18T02:00:00Z', '2026-11-18'), 0);
});
