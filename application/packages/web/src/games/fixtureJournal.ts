/**
 * Bundled sample journal for Plan step 6 (and demos before a user picks a folder).
 * Keep in sync with `application/tools/fixtures/journal/`.
 */

import type { JournalGame, JournalSession, ReturnCardModel } from './returnCard';
import { listGames, returnCardFor } from './returnCard';

/** Tiny 1×1 PNG (same bytes as tools/fixtures/journal/frames/frame-fixture-1.png). */
export const FIXTURE_FRAME_PNG_BASE64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';

export const FIXTURE_GAME_ID = 'game-fixture-1';

export const FIXTURE_GAMES: JournalGame[] = [
  {
    id: FIXTURE_GAME_ID,
    revision_id: 'rev-root',
    parent_revision_id: null,
    device_id: 'device-fixture',
    title: 'Fixture Souls',
    executables: [],
    cover_path: null,
    come_back_on: null,
  },
];

export const FIXTURE_SESSIONS: JournalSession[] = [
  {
    id: 'session-fixture-1',
    device_id: 'device-fixture',
    game_id: FIXTURE_GAME_ID,
    started_at: '2026-11-11T01:00:00Z',
    stopped_at: '2026-11-11T02:30:00Z',
    text: 'cleared the chapel',
    what_next: 'open the side door',
    frame_id: 'frame-fixture-1',
  },
];

/** Default "today" for the fixture return card (7 days after the session). */
export const FIXTURE_AS_OF_DAY = '2026-11-18';

export function fixtureReturnCard(asOfDay: string = FIXTURE_AS_OF_DAY): ReturnCardModel {
  const games = listGames(FIXTURE_GAMES);
  const game = games.find((g) => g.id === FIXTURE_GAME_ID);
  if (!game) throw new Error('fixture game missing');
  return returnCardFor(game, FIXTURE_SESSIONS, asOfDay);
}

export function fixtureFrameDataUrl(): string {
  return `data:image/png;base64,${FIXTURE_FRAME_PNG_BASE64}`;
}
