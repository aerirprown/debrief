/**
 * Derive the return card from journal sessions + a game record.
 * Mirrors `debrief-journal::return_card` for the web shell and KATs.
 */

export type JournalSession = {
  id: string;
  device_id: string;
  game_id: string;
  started_at: string;
  stopped_at: string;
  text?: string;
  what_next?: string;
  frame_id?: string | null;
};

export type JournalGame = {
  id: string;
  revision_id: string;
  parent_revision_id?: string | null;
  device_id: string;
  title: string;
  executables?: string[];
  cover_path?: string | null;
  come_back_on?: string | null;
};

export type ReturnCardModel = {
  gameId: string;
  title: string;
  /** Last session line ("what happened"). */
  line: string;
  whatNext: string;
  /** Whole UTC days between stop day and asOfDay. */
  daysSince: number;
  frameId: string | null;
  /** Relative path under the journal root, when a frame exists. */
  framePath: string | null;
  sessionId: string | null;
  stoppedAt: string | null;
};

export function newestSession(
  sessions: JournalSession[],
  gameId: string,
): JournalSession | null {
  let best: JournalSession | null = null;
  for (const s of sessions) {
    if (s.game_id !== gameId) continue;
    if (
      !best
      || s.stopped_at > best.stopped_at
      || (s.stopped_at === best.stopped_at && s.started_at > best.started_at)
      || (s.stopped_at === best.stopped_at
        && s.started_at === best.started_at
        && s.id > best.id)
    ) {
      best = s;
    }
  }
  return best;
}

/** Civil day number (Howard Hinnant) for YYYY-MM-DD. */
function dayNumber(ymd: string): number | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(ymd);
  if (!m) return null;
  let y = Number(m[1]);
  const month = Number(m[2]);
  const d = Number(m[3]);
  if (month < 1 || month > 12 || d < 1 || d > 31) return null;
  if (month <= 2) y -= 1;
  const era = Math.floor(y / 400);
  const yoe = y - era * 400;
  const mpb = month > 2 ? month - 3 : month + 9;
  const doy = Math.floor((153 * mpb + 2) / 5) + d - 1;
  const doe = yoe * 365 + Math.floor(yoe / 4) - Math.floor(yoe / 100) + doy;
  return era * 146097 + doe - 719468;
}

export function daysSinceStopped(stoppedAt: string, asOfDay: string): number {
  const t = stoppedAt.indexOf('T');
  const stopDay = t >= 0 ? stoppedAt.slice(0, t) : stoppedAt.slice(0, 10);
  const stop = dayNumber(stopDay);
  const asOf = dayNumber(asOfDay);
  if (stop == null || asOf == null || asOf <= stop) return 0;
  return asOf - stop;
}

export function returnCardFor(
  game: JournalGame,
  sessions: JournalSession[],
  asOfDay: string,
): ReturnCardModel {
  const session = newestSession(sessions, game.id);
  const frameId = session?.frame_id ?? null;
  return {
    gameId: game.id,
    title: game.title,
    line: session?.text ?? '',
    whatNext: session?.what_next ?? '',
    daysSince: session ? daysSinceStopped(session.stopped_at, asOfDay) : 0,
    frameId,
    framePath: frameId ? `frames/${frameId}.png` : null,
    sessionId: session?.id ?? null,
    stoppedAt: session?.stopped_at ?? null,
  };
}

/** Pick newest unforked game revision per id (same rule as the Rust crate). */
export function listGames(revisions: JournalGame[]): JournalGame[] {
  const byId = new Map<string, JournalGame[]>();
  for (const g of revisions) {
    const list = byId.get(g.id) ?? [];
    list.push(g);
    byId.set(g.id, list);
  }
  const tips: JournalGame[] = [];
  for (const group of byId.values()) {
    const tip = newestUnforkedGame(group);
    if (tip) tips.push(tip);
  }
  tips.sort((a, b) => a.title.localeCompare(b.title) || a.id.localeCompare(b.id));
  return tips;
}

function newestUnforkedGame(revisions: JournalGame[]): JournalGame | null {
  if (!revisions.length) return null;
  const childCount = new Map<string | null, number>();
  for (const r of revisions) {
    const key = r.parent_revision_id ?? null;
    childCount.set(key, (childCount.get(key) ?? 0) + 1);
  }
  for (const n of childCount.values()) {
    if (n > 1) return null;
  }
  const named = new Set(
    revisions.map((r) => r.parent_revision_id).filter((p): p is string => !!p),
  );
  const tips = revisions.filter((r) => !named.has(r.revision_id));
  return tips.length === 1 ? tips[0]! : null;
}
