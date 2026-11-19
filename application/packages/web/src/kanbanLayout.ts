/**
 * The layout a new Kanban board starts with, and the empty body of a new
 * Kanban task. No editor code here: the New menus and the empty-draft check
 * read it without loading the editor.
 * Spec: docs/internal/issues/0270.md (Spec)
 */

import { readKanbanPref } from './editorPrefs';
import { isColumnColor } from './calloutKeywords';

type Translate = (key: string) => string;

/** The color each slot of a new board starts with. */
export const DEFAULT_COLORS: Record<3 | 4, string[]> = {
  3: ['purple', 'blue', 'green'],
  4: ['purple', 'blue', 'orange', 'green'],
};

function slotKey(count: number, slot: number): string {
  const keys = count === 4 ? ['todo', 'inProgress', 'review', 'done'] : ['todo', 'inProgress', 'done'];
  return keys[slot] ?? keys[keys.length - 1]!;
}

/** The default name of `slot` in a board of `count` columns. */
export function slotName(t: Translate, count: number, slot: number): string {
  return t(`editor:kanban.${slotKey(count, slot)}`);
}

/** The look icon each default slot starts with, before its name in the
 *  title (`:icon[eye] Review`). GitHub #752. */
const SLOT_ICONS: Record<string, string> = {
  todo: 'clipboard-text',
  inProgress: 'person-simple-run',
  review: 'eye',
  done: 'check-circle',
};

/** The icon of `slot` in a board of `count` columns. */
export function slotIcon(count: number, slot: number): string {
  return SLOT_ICONS[slotKey(count, slot)]!;
}

/** A new board: the count and colors of the last board on this device, the
 *  default names in the reader's language. */
export function startLayout(t: Translate): { names: string[]; colors: string[]; icons: string[] } {
  const pref = readKanbanPref();
  const count = pref?.count ?? 3;
  const colors = DEFAULT_COLORS[count].map((c, i) => (isColumnColor(pref?.colors[i]) ? pref!.colors[i]! : c));
  const slots = Array.from({ length: count }, (_, i) => i);
  return { names: slots.map((i) => slotName(t, count, i)), colors, icons: slots.map((i) => slotIcon(count, i)) };
}

/** The body a new Kanban task starts with, in the form the editor saves an
 *  untouched board, so the draft check below can tell it from content. */
export function kanbanEmptyBody(t: Translate): string {
  const { names, colors, icons } = startLayout(t);
  const columns = names.map((name, i) => `> > [!column|${colors[i]}]+ :icon[${icons[i]}] ${name}\n> > - [ ] &nbsp;`).join('\n>\n');
  return `> [!kanban]+\n${columns}\n\n&nbsp;`;
}

const UNTOUCHED_KANBAN = /^> \[!kanban\]\+\n(?:> > \[!column(?:\|[\w-]+)?\]\+[^\n]*\n> > - \[ \] &nbsp;(?:\n>\n|\n\n))+&nbsp;$/;

/** A board with only its column titles and one empty card line each: the
 *  body of a Kanban task nobody wrote in yet. */
export function isUntouchedKanbanBody(body: string): boolean {
  return UNTOUCHED_KANBAN.test(body);
}

const COLUMN_START = /^> > \[!column(?:\|[\w-]+)?\]\+/;
const CARD_LINE = /^> > - \[([ xX])\] (.*)$/;

type BoardColumn = { header: string; cards: string[] };

/** Split a kanban markdown body into columns and card titles (placeholders omitted). */
export function parseBoardColumns(body: string): BoardColumn[] | null {
  if (!body.includes('[!kanban]')) return null;
  const lines = body.replace(/\r\n/g, '\n').split('\n');
  const columns: BoardColumn[] = [];
  let current: BoardColumn | null = null;
  for (const line of lines) {
    if (COLUMN_START.test(line)) {
      current = { header: line, cards: [] };
      columns.push(current);
      continue;
    }
    if (!current) continue;
    const m = CARD_LINE.exec(line);
    if (!m) continue;
    const text = (m[2] ?? '').replace(/&nbsp;/g, ' ').trim();
    if (!text) continue;
    current.cards.push(text);
  }
  return columns.length ? columns : null;
}

function rebuildBoard(body: string, columns: BoardColumn[]): string {
  const lines = body.replace(/\r\n/g, '\n').split('\n');
  const out: string[] = [];
  let col = -1;
  let writingCards = false;
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (COLUMN_START.test(line)) {
      if (writingCards && col >= 0) {
        /* cards already emitted */
      }
      col++;
      writingCards = true;
      out.push(line);
      const cards = columns[col]?.cards ?? [];
      if (cards.length === 0) out.push('> > - [ ] &nbsp;');
      else for (const title of cards) out.push(`> > - [ ] ${title}`);
      continue;
    }
    if (writingCards && CARD_LINE.test(line)) continue;
    if (writingCards && (line === '>' || line === '')) {
      writingCards = false;
      out.push(line);
      continue;
    }
    if (writingCards && !COLUMN_START.test(line) && !CARD_LINE.test(line)) {
      writingCards = false;
    }
    if (!writingCards || !CARD_LINE.test(line)) out.push(line);
  }
  return out.join('\n');
}

/** Add a titled card to a column (0-based). Replaces the empty placeholder. */
export function addCardToBoard(body: string, columnIndex: number, title: string): string | null {
  const columns = parseBoardColumns(body);
  if (!columns || columnIndex < 0 || columnIndex >= columns.length) return null;
  const name = title.trim();
  if (!name) return null;
  columns[columnIndex]!.cards.push(name);
  return rebuildBoard(body, columns);
}

/** Move a card from one column to another by card index within the source. */
export function moveCardOnBoard(
  body: string,
  fromColumn: number,
  cardIndex: number,
  toColumn: number,
): string | null {
  const columns = parseBoardColumns(body);
  if (!columns) return null;
  if (fromColumn < 0 || fromColumn >= columns.length) return null;
  if (toColumn < 0 || toColumn >= columns.length) return null;
  const source = columns[fromColumn]!;
  if (cardIndex < 0 || cardIndex >= source.cards.length) return null;
  const [card] = source.cards.splice(cardIndex, 1);
  if (card === undefined) return null;
  columns[toColumn]!.cards.push(card);
  return rebuildBoard(body, columns);
}
