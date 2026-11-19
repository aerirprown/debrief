/**
 * Library note body for a game page in the folder tree.
 * Journal data (sessions, frames) lives in the gaming journal folder;
 * this body only points at a game id.
 */

export const GAME_PAGE_TYPE = 'debrief-game' as const;

export type GamePageBody = {
  type: typeof GAME_PAGE_TYPE;
  version: number;
  gameId: string;
};

export function isGamePageBody(body: string | undefined | null): boolean {
  if (body == null) return false;
  try {
    const data: unknown = JSON.parse(body);
    return (
      typeof data === 'object'
      && data !== null
      && (data as { type?: unknown }).type === GAME_PAGE_TYPE
      && typeof (data as { gameId?: unknown }).gameId === 'string'
    );
  } catch {
    return false;
  }
}

export function parseGamePageBody(body: string): GamePageBody | null {
  if (!isGamePageBody(body)) return null;
  const data = JSON.parse(body) as GamePageBody;
  return {
    type: GAME_PAGE_TYPE,
    version: typeof data.version === 'number' ? data.version : 1,
    gameId: data.gameId,
  };
}

export function emptyGamePageBody(gameId: string): string {
  return JSON.stringify({
    type: GAME_PAGE_TYPE,
    version: 1,
    gameId,
  } satisfies GamePageBody);
}
