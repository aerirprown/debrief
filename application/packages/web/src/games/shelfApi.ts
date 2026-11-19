/** Tauri shelf commands (Steam + manual games). */

import { isPhone } from '../autoBackup/destination';

export type ShelfRow = {
  id: string;
  title: string;
  source: 'steam' | 'manual' | string;
  steamAppId?: number | null;
  installDir?: string | null;
  coverPath?: string | null;
  line: string;
  daysSince: number;
  canLaunch: boolean;
  launchUri?: string | null;
  launchExe?: string | null;
};

export type SessionRow = {
  id: string;
  deviceId: string;
  gameId: string;
  startedAt: string;
  stoppedAt: string;
  text: string;
  whatNext: string;
  frameId?: string | null;
};

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

export async function listShelf(opts: {
  journalRoot: string;
  steamRoot?: string | null;
  asOfDay: string;
  phone?: boolean;
}): Promise<ShelfRow[]> {
  const phone = opts.phone ?? isPhone();
  return invoke<ShelfRow[]>('shelf_list', {
    journalRoot: opts.journalRoot,
    steamRoot: phone ? null : (opts.steamRoot ?? null),
    asOfDay: opts.asOfDay,
    phone,
  });
}

export async function addManualGame(opts: {
  journalRoot: string;
  title: string;
  executable?: string;
  coverFile?: string;
  steamAppId?: number;
}): Promise<ShelfRow> {
  return invoke<ShelfRow>('shelf_add_manual', {
    journalRoot: opts.journalRoot,
    input: {
      title: opts.title,
      executable: opts.executable,
      coverFile: opts.coverFile,
      steamAppId: opts.steamAppId,
    },
  });
}

export async function addManualSession(opts: {
  journalRoot: string;
  gameId: string;
  text: string;
  whatNext?: string;
  startedAt?: string;
  stoppedAt?: string;
}): Promise<SessionRow> {
  const now = new Date().toISOString();
  return invoke<SessionRow>('journal_add_session', {
    journalRoot: opts.journalRoot,
    input: {
      gameId: opts.gameId,
      text: opts.text,
      whatNext: opts.whatNext ?? '',
      startedAt: opts.startedAt ?? now,
      stoppedAt: opts.stoppedAt ?? now,
    },
  });
}

export async function launchShelfGame(row: ShelfRow): Promise<void> {
  if (isPhone()) {
    throw new Error('launch is desktop-only');
  }
  await invoke('shelf_launch', {
    launchUri: row.launchUri ?? null,
    launchExe: row.launchExe ?? null,
  });
}

export async function defaultSteamRoot(): Promise<string | null> {
  if (isPhone()) return null;
  try {
    return await invoke<string | null>('shelf_default_steam_root');
  } catch {
    return null;
  }
}

export async function ensureJournal(journalRoot: string): Promise<string> {
  return invoke<string>('shelf_ensure_journal', { journalRoot });
}

export async function listJournalFiles(journalRoot: string): Promise<string[]> {
  return invoke<string[]>('journal_list_files', { journalRoot });
}

export async function pickJournalFolder(): Promise<string | null> {
  try {
    return await invoke<string | null>('journal_pick_folder');
  } catch {
    return null;
  }
}

export async function installDeviceId(): Promise<string | null> {
  try {
    return await invoke<string>('journal_install_device_id');
  } catch {
    return null;
  }
}

export function todayIsoDay(d = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

export const JOURNAL_ROOT_STORAGE_KEY = 'debrief.journalRoot';
