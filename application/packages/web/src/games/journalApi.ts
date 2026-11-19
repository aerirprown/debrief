/**
 * Tauri invoke helpers for the gaming journal. Falls back silently when not
 * running inside the desktop/mobile shell (tests / plain web).
 */

import type { ReturnCardModel } from './returnCard';

export type JournalGameDto = {
  id: string;
  title: string;
  coverPath?: string | null;
  comeBackOn?: string | null;
};

type ReturnCardDto = {
  gameId: string;
  title: string;
  line: string;
  whatNext: string;
  daysSince: number;
  frameId?: string | null;
  framePath?: string | null;
  sessionId?: string | null;
  stoppedAt?: string | null;
};

async function invoke<T>(cmd: string, args: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return await tauriInvoke<T>(cmd, args);
  } catch {
    return null;
  }
}

export async function listJournalGames(journalRoot: string): Promise<JournalGameDto[]> {
  const rows = await invoke<JournalGameDto[]>('journal_list_games', { journalRoot });
  return rows ?? [];
}

export async function fetchReturnCard(
  journalRoot: string,
  gameId: string,
  asOfDay: string,
): Promise<ReturnCardModel | null> {
  const dto = await invoke<ReturnCardDto | null>('journal_return_card', {
    journalRoot,
    gameId,
    asOfDay,
  });
  if (!dto) return null;
  return {
    gameId: dto.gameId,
    title: dto.title,
    line: dto.line,
    whatNext: dto.whatNext,
    daysSince: dto.daysSince,
    frameId: dto.frameId ?? null,
    framePath: dto.framePath ?? null,
    sessionId: dto.sessionId ?? null,
    stoppedAt: dto.stoppedAt ?? null,
  };
}
