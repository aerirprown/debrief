/**
 * Freeform canvas page body — OneNote-style containers with position/size.
 * Shared shape for ordinary notes and (later) game note pages.
 *
 * Body is JSON `{ type: 'debrief-freeform', … }`. Each save that changes
 * content mints a new revisionId and names the previous as parentRevisionId
 * (fork rules match the gaming journal; step 5 writes those as files).
 */

import { isRecord } from '../isRecord';

export const FREEFORM_TYPE = 'debrief-freeform' as const;

export type FreeformContainerKind = 'text' | 'image' | 'table' | 'audio';

export type FreeformTextContainer = {
  id: string;
  kind: 'text';
  x: number;
  y: number;
  w: number;
  h: number;
  /** Inline HTML for the text box (bold/italic/lists/links). */
  html: string;
};

export type FreeformImageContainer = {
  id: string;
  kind: 'image';
  x: number;
  y: number;
  w: number;
  h: number;
  /** `pn:img/<uuid>` — bytes live in the encrypted image store. */
  src: string;
};

export type FreeformTableContainer = {
  id: string;
  kind: 'table';
  x: number;
  y: number;
  w: number;
  h: number;
  rows: string[][];
};

export type FreeformAudioContainer = {
  id: string;
  kind: 'audio';
  x: number;
  y: number;
  w: number;
  h: number;
  /** `audio/{id}.ext` in the gaming journal, or a blob/object URL for local notes. */
  src: string;
};

export type FreeformContainer =
  | FreeformTextContainer
  | FreeformImageContainer
  | FreeformTableContainer
  | FreeformAudioContainer;

export type FreeformReminder = {
  id: string;
  /** RFC 3339 UTC. */
  at: string;
  label?: string | null;
  containerId?: string | null;
  cleared: boolean;
};

export type FreeformPage = {
  type: typeof FREEFORM_TYPE;
  version: number;
  revisionId: string;
  parentRevisionId: string | null;
  containers: FreeformContainer[];
  reminders: FreeformReminder[];
};

const DEFAULT_TEXT_W = 280;
const DEFAULT_TEXT_H = 96;
const DEFAULT_IMAGE_W = 240;
const DEFAULT_IMAGE_H = 180;
const DEFAULT_TABLE_W = 320;
const DEFAULT_TABLE_H = 120;
const DEFAULT_AUDIO_W = 280;
const DEFAULT_AUDIO_H = 56;

function newId(): string {
  return crypto.randomUUID();
}

export function emptyPage(): FreeformPage {
  return {
    type: FREEFORM_TYPE,
    version: 1,
    revisionId: newId(),
    parentRevisionId: null,
    containers: [],
    reminders: [],
  };
}

/** Body string a new freeform note is born with. */
export function emptyFreeformBody(): string {
  return serializePage(emptyPage());
}

export function isFreeformBody(body: string | undefined | null): boolean {
  if (body == null) return false;
  const trimmed = body.trim();
  if (!trimmed) return true;
  try {
    const data: unknown = JSON.parse(trimmed);
    return isRecord(data) && data.type === FREEFORM_TYPE;
  } catch {
    return false;
  }
}

export function isEmptyPage(page: FreeformPage): boolean {
  return page.containers.length === 0;
}

/** Plain text for search / list excerpts. */
export function pageText(page: FreeformPage): string {
  const parts: string[] = [];
  for (const c of page.containers) {
    if (c.kind === 'text') {
      const plain = htmlToPlain(c.html).trim();
      if (plain) parts.push(plain);
    } else if (c.kind === 'table') {
      for (const row of c.rows) {
        const line = row.map((cell) => cell.trim()).filter(Boolean).join('\t');
        if (line) parts.push(line);
      }
    }
  }
  return parts.join('\n');
}

export function htmlToPlain(html: string): string {
  return html
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/p>/gi, '\n')
    .replace(/<\/li>/gi, '\n')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"');
}

/**
 * Parse a body into a page. Empty string → empty page. Non-freeform JSON or
 * markdown → null (caller keeps the TipTap document editor).
 */
export function parsePage(body: string): FreeformPage | null {
  if (!body.trim()) return emptyPage();
  let data: unknown;
  try {
    data = JSON.parse(body);
  } catch {
    return null;
  }
  if (!isRecord(data) || data.type !== FREEFORM_TYPE) return null;
  const containers: FreeformContainer[] = [];
  if (Array.isArray(data.containers)) {
    for (const raw of data.containers) {
      const c = parseContainer(raw);
      if (c) containers.push(c);
    }
  }
  const reminders: FreeformReminder[] = [];
  if (Array.isArray(data.reminders)) {
    for (const raw of data.reminders) {
      const r = parseReminder(raw);
      if (r) reminders.push(r);
    }
  }
  return {
    type: FREEFORM_TYPE,
    version: typeof data.version === 'number' ? data.version : 1,
    revisionId: typeof data.revisionId === 'string' ? data.revisionId : newId(),
    parentRevisionId: typeof data.parentRevisionId === 'string' ? data.parentRevisionId : null,
    containers,
    reminders,
  };
}

function parseReminder(raw: unknown): FreeformReminder | null {
  if (!isRecord(raw) || typeof raw.id !== 'string' || typeof raw.at !== 'string') return null;
  return {
    id: raw.id,
    at: raw.at,
    label: typeof raw.label === 'string' ? raw.label : null,
    containerId: typeof raw.containerId === 'string'
      ? raw.containerId
      : typeof raw.container_id === 'string'
        ? raw.container_id
        : null,
    cleared: raw.cleared === true,
  };
}

function parseContainer(raw: unknown): FreeformContainer | null {
  if (!isRecord(raw) || typeof raw.id !== 'string') return null;
  const x = num(raw.x, 0);
  const y = num(raw.y, 0);
  const w = Math.max(40, num(raw.w, DEFAULT_TEXT_W));
  const h = Math.max(40, num(raw.h, DEFAULT_TEXT_H));
  const kind = raw.kind;
  if (kind === 'text') {
    return { id: raw.id, kind: 'text', x, y, w, h, html: typeof raw.html === 'string' ? raw.html : '' };
  }
  if (kind === 'image') {
    if (typeof raw.src !== 'string' || !raw.src) return null;
    return {
      id: raw.id,
      kind: 'image',
      x,
      y,
      w: Math.max(40, num(raw.w, DEFAULT_IMAGE_W)),
      h: Math.max(40, num(raw.h, DEFAULT_IMAGE_H)),
      src: raw.src,
    };
  }
  if (kind === 'table') {
    const rows = parseRows(raw.rows);
    return {
      id: raw.id,
      kind: 'table',
      x,
      y,
      w: Math.max(80, num(raw.w, DEFAULT_TABLE_W)),
      h: Math.max(40, num(raw.h, DEFAULT_TABLE_H)),
      rows,
    };
  }
  if (kind === 'audio') {
    if (typeof raw.src !== 'string' || !raw.src) return null;
    return {
      id: raw.id,
      kind: 'audio',
      x,
      y,
      w: Math.max(80, num(raw.w, DEFAULT_AUDIO_W)),
      h: Math.max(40, num(raw.h, DEFAULT_AUDIO_H)),
      src: raw.src,
    };
  }
  return null;
}

function parseRows(raw: unknown): string[][] {
  if (!Array.isArray(raw) || raw.length === 0) return [['']];
  return raw.map((row) => {
    if (!Array.isArray(row)) return [''];
    return row.map((cell) => (typeof cell === 'string' ? cell : String(cell ?? '')));
  });
}

function num(v: unknown, fallback: number): number {
  return typeof v === 'number' && Number.isFinite(v) ? v : fallback;
}

export function serializePage(page: FreeformPage): string {
  return JSON.stringify({
    type: FREEFORM_TYPE,
    version: page.version,
    revisionId: page.revisionId,
    parentRevisionId: page.parentRevisionId,
    containers: page.containers,
    reminders: page.reminders ?? [],
  });
}

/** Mint a new revision that names the current one as its parent. */
export function nextRevision(page: FreeformPage): FreeformPage {
  return {
    ...page,
    reminders: page.reminders ?? [],
    revisionId: newId(),
    parentRevisionId: page.revisionId,
  };
}

export function addTextBox(page: FreeformPage, x: number, y: number, html = ''): FreeformPage {
  const box: FreeformTextContainer = {
    id: newId(),
    kind: 'text',
    x: Math.max(0, Math.round(x)),
    y: Math.max(0, Math.round(y)),
    w: DEFAULT_TEXT_W,
    h: DEFAULT_TEXT_H,
    html,
  };
  return { ...nextRevision(page), containers: [...page.containers, box] };
}

export function addImage(
  page: FreeformPage,
  x: number,
  y: number,
  src: string,
  w = DEFAULT_IMAGE_W,
  h = DEFAULT_IMAGE_H,
): FreeformPage {
  const box: FreeformImageContainer = {
    id: newId(),
    kind: 'image',
    x: Math.max(0, Math.round(x)),
    y: Math.max(0, Math.round(y)),
    w,
    h,
    src,
  };
  return { ...nextRevision(page), containers: [...page.containers, box] };
}

export function addTable(page: FreeformPage, x: number, y: number, rows: string[][]): FreeformPage {
  const cols = Math.max(1, ...rows.map((r) => r.length));
  const normalized = rows.map((r) => {
    const next = [...r];
    while (next.length < cols) next.push('');
    return next;
  });
  const box: FreeformTableContainer = {
    id: newId(),
    kind: 'table',
    x: Math.max(0, Math.round(x)),
    y: Math.max(0, Math.round(y)),
    w: Math.max(DEFAULT_TABLE_W, cols * 96),
    h: Math.max(DEFAULT_TABLE_H, normalized.length * 32 + 16),
    rows: normalized.length ? normalized : [['']],
  };
  return { ...nextRevision(page), containers: [...page.containers, box] };
}

export function addAudio(
  page: FreeformPage,
  x: number,
  y: number,
  src: string,
  w = DEFAULT_AUDIO_W,
  h = DEFAULT_AUDIO_H,
): FreeformPage {
  const box: FreeformAudioContainer = {
    id: newId(),
    kind: 'audio',
    x: Math.max(0, Math.round(x)),
    y: Math.max(0, Math.round(y)),
    w,
    h,
    src,
  };
  return { ...nextRevision(page), containers: [...page.containers, box] };
}

export function addReminder(
  page: FreeformPage,
  at: string,
  label?: string | null,
  containerId?: string | null,
): FreeformPage {
  const rem: FreeformReminder = {
    id: newId(),
    at,
    label: label ?? null,
    containerId: containerId ?? null,
    cleared: false,
  };
  return {
    ...nextRevision(page),
    reminders: [...(page.reminders ?? []), rem],
  };
}

export function clearReminder(page: FreeformPage, reminderId: string): FreeformPage {
  const reminders = (page.reminders ?? []).map((r) =>
    r.id === reminderId ? { ...r, cleared: true } : r,
  );
  if (JSON.stringify(reminders) === JSON.stringify(page.reminders ?? [])) return page;
  return { ...nextRevision(page), reminders };
}

export type FreeformContainerPatch = {
  x?: number;
  y?: number;
  w?: number;
  h?: number;
  html?: string;
  src?: string;
  rows?: string[][];
};

export function updateContainer(page: FreeformPage, id: string, patch: FreeformContainerPatch): FreeformPage {
  let changed = false;
  const containers = page.containers.map((c) => {
    if (c.id !== id) return c;
    if (c.kind === 'text') {
      const next: FreeformTextContainer = {
        ...c,
        x: patch.x ?? c.x,
        y: patch.y ?? c.y,
        w: patch.w ?? c.w,
        h: patch.h ?? c.h,
        html: patch.html ?? c.html,
      };
      if (next.x === c.x && next.y === c.y && next.w === c.w && next.h === c.h && next.html === c.html) return c;
      changed = true;
      return next;
    }
    if (c.kind === 'image') {
      const next: FreeformImageContainer = {
        ...c,
        x: patch.x ?? c.x,
        y: patch.y ?? c.y,
        w: patch.w ?? c.w,
        h: patch.h ?? c.h,
        src: patch.src ?? c.src,
      };
      if (next.x === c.x && next.y === c.y && next.w === c.w && next.h === c.h && next.src === c.src) return c;
      changed = true;
      return next;
    }
    if (c.kind === 'audio') {
      const next: FreeformAudioContainer = {
        ...c,
        x: patch.x ?? c.x,
        y: patch.y ?? c.y,
        w: patch.w ?? c.w,
        h: patch.h ?? c.h,
        src: patch.src ?? c.src,
      };
      if (next.x === c.x && next.y === c.y && next.w === c.w && next.h === c.h && next.src === c.src) return c;
      changed = true;
      return next;
    }
    const next: FreeformTableContainer = {
      ...c,
      x: patch.x ?? c.x,
      y: patch.y ?? c.y,
      w: patch.w ?? c.w,
      h: patch.h ?? c.h,
      rows: patch.rows ?? c.rows,
    };
    if (
      next.x === c.x &&
      next.y === c.y &&
      next.w === c.w &&
      next.h === c.h &&
      JSON.stringify(next.rows) === JSON.stringify(c.rows)
    ) {
      return c;
    }
    changed = true;
    return next;
  });
  if (!changed) return page;
  return { ...nextRevision(page), containers };
}

export function moveContainer(page: FreeformPage, id: string, x: number, y: number): FreeformPage {
  return updateContainer(page, id, { x: Math.max(0, Math.round(x)), y: Math.max(0, Math.round(y)) });
}

export function removeContainer(page: FreeformPage, id: string): FreeformPage {
  if (!page.containers.some((c) => c.id === id)) return page;
  return {
    ...nextRevision(page),
    containers: page.containers.filter((c) => c.id !== id),
  };
}

/** Parse an HTML table or tab-separated clipboard text into rows. */
export function rowsFromClipboard(html: string, text: string): string[][] | null {
  const fromHtml = rowsFromHtmlTable(html);
  if (fromHtml) return fromHtml;
  const trimmed = text.replace(/\r\n/g, '\n').trim();
  if (!trimmed) return null;
  if (!trimmed.includes('\t') && !trimmed.includes('\n')) return null;
  const rows = trimmed.split('\n').map((line) => line.split('\t'));
  if (rows.length === 1 && rows[0]!.length === 1) return null;
  return rows;
}

function rowsFromHtmlTable(html: string): string[][] | null {
  if (!html || !/<table[\s>]/i.test(html)) return null;
  try {
    const doc = new DOMParser().parseFromString(html, 'text/html');
    const table = doc.querySelector('table');
    if (!table) return null;
    const rows: string[][] = [];
    for (const tr of table.querySelectorAll('tr')) {
      const cells: string[] = [];
      for (const cell of tr.querySelectorAll('th, td')) {
        cells.push((cell.textContent ?? '').trim());
      }
      if (cells.length) rows.push(cells);
    }
    return rows.length ? rows : null;
  } catch {
    return null;
  }
}

export const FREEFORM_DEFAULTS = {
  textW: DEFAULT_TEXT_W,
  textH: DEFAULT_TEXT_H,
  imageW: DEFAULT_IMAGE_W,
  imageH: DEFAULT_IMAGE_H,
  tableW: DEFAULT_TABLE_W,
  tableH: DEFAULT_TABLE_H,
  audioW: DEFAULT_AUDIO_W,
  audioH: DEFAULT_AUDIO_H,
} as const;
