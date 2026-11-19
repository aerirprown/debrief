/** Tauri commands for game note extras (hotkey, voice, reminders). */

export type PendingNotify = {
  id: string;
  kind: 'reminder' | 'comeBack' | string;
  gameId: string;
  noteId?: string | null;
  containerId?: string | null;
  at: string;
  title: string;
  body: string;
  due: boolean;
};

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

export async function appendHotkeyLine(opts: {
  journalRoot: string;
  gameId: string;
  line: string;
}): Promise<void> {
  await invoke('note_append_hotkey_line', {
    journalRoot: opts.journalRoot,
    gameId: opts.gameId,
    line: opts.line,
  });
}

export async function writeVoiceMemo(opts: {
  journalRoot: string;
  bytes: Uint8Array;
  extension: string;
}): Promise<string> {
  return invoke<string>('note_write_audio', {
    journalRoot: opts.journalRoot,
    bytes: Array.from(opts.bytes),
    extension: opts.extension,
  });
}

export async function audioFilePath(opts: {
  journalRoot: string;
  src: string;
}): Promise<string | null> {
  return invoke<string | null>('note_audio_path', {
    journalRoot: opts.journalRoot,
    src: opts.src,
  });
}

export async function notificationPlan(opts: {
  journalRoot: string;
  now: string;
}): Promise<PendingNotify[]> {
  return invoke<PendingNotify[]>('note_notification_plan', {
    journalRoot: opts.journalRoot,
    now: opts.now,
  });
}

export async function dueNotifications(opts: {
  journalRoot: string;
  now: string;
}): Promise<PendingNotify[]> {
  return invoke<PendingNotify[]>('note_due_notifications', {
    journalRoot: opts.journalRoot,
    now: opts.now,
  });
}

/** Fire due journal notifications via the OS notification plugin. */
export async function fireDueJournalNotifications(journalRoot: string): Promise<number> {
  const now = new Date().toISOString();
  const due = await dueNotifications({ journalRoot, now });
  if (due.length === 0) return 0;
  try {
    const mod = await import('@tauri-apps/plugin-notification');
    const granted = await mod.isPermissionGranted();
    if (!granted) {
      const perm = await mod.requestPermission();
      if (perm !== 'granted') return 0;
    }
    for (const n of due) {
      await mod.sendNotification({ title: n.title, body: n.body });
    }
  } catch {
    return 0;
  }
  return due.length;
}

/**
 * Schedule future come-back / note reminders as local notifications (phones).
 * Each install schedules from its own journal folder; nothing is pushed.
 */
export async function scheduleJournalNotifications(journalRoot: string): Promise<number> {
  const now = new Date().toISOString();
  const plan = await notificationPlan({ journalRoot, now });
  const future = plan.filter((n) => !n.due && n.at > now);
  if (future.length === 0) {
    return fireDueJournalNotifications(journalRoot);
  }
  try {
    const mod = await import('@tauri-apps/plugin-notification');
    const granted = await mod.isPermissionGranted();
    if (!granted) {
      const perm = await mod.requestPermission();
      if (perm !== 'granted') return 0;
    }
    let scheduled = 0;
    for (const n of future) {
      const at = new Date(n.at);
      if (Number.isNaN(at.getTime())) continue;
      await mod.sendNotification({
        title: n.title,
        body: n.body,
        schedule: mod.Schedule.at(at, false, true),
      });
      scheduled += 1;
    }
    scheduled += await fireDueJournalNotifications(journalRoot);
    return scheduled;
  } catch {
    return fireDueJournalNotifications(journalRoot);
  }
}

export function nowRfc3339(d = new Date()): string {
  return d.toISOString();
}
