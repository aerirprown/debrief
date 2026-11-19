/**
 * Start/stop the desktop game watcher (Windows / Linux).
 */

export type WatchGameInput = {
  gameId: string;
  title: string;
  matchPaths: string[];
};

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
    return await tauriInvoke<T>(cmd, args);
  } catch {
    return null;
  }
}

export async function startWatcher(opts: {
  journalRoot: string;
  deviceId: string;
  games: WatchGameInput[];
  debounceMs?: number;
  graceMs?: number;
}): Promise<boolean> {
  const ok = await invoke('watcher_start', {
    journalRoot: opts.journalRoot,
    deviceId: opts.deviceId,
    games: opts.games,
    debounceMs: opts.debounceMs,
    graceMs: opts.graceMs,
  });
  return ok !== null;
}

export async function stopWatcher(): Promise<void> {
  await invoke('watcher_stop');
}
