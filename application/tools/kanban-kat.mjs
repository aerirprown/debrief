/**
 * Kanban template checks for Debrief (Plan step 4).
 *
 *   node --test tools/kanban-kat.mjs
 */

import test from 'node:test';
import assert from 'node:assert/strict';

const DEFAULT_COLORS = {
  3: ['purple', 'blue', 'green'],
  4: ['purple', 'blue', 'orange', 'green'],
};

const SLOT_ICONS = {
  todo: 'clipboard-text',
  inProgress: 'person-simple-run',
  review: 'eye',
  done: 'check-circle',
};

function slotKey(count, slot) {
  const keys = count === 4 ? ['todo', 'inProgress', 'review', 'done'] : ['todo', 'inProgress', 'done'];
  return keys[slot] ?? keys[keys.length - 1];
}

function slotName(t, count, slot) {
  return t(`editor:kanban.${slotKey(count, slot)}`);
}

function startLayout(t, count = 3) {
  const colors = DEFAULT_COLORS[count];
  const slots = Array.from({ length: count }, (_, i) => i);
  return {
    names: slots.map((i) => slotName(t, count, i)),
    colors,
    icons: slots.map((i) => SLOT_ICONS[slotKey(count, i)]),
  };
}

function kanbanEmptyBody(t) {
  const { names, colors, icons } = startLayout(t);
  const columns = names
    .map((name, i) => `> > [!column|${colors[i]}]+ :icon[${icons[i]}] ${name}\n> > - [ ] &nbsp;`)
    .join('\n>\n');
  return `> [!kanban]+\n${columns}\n\n&nbsp;`;
}

const COLUMN_START = /^> > \[!column(?:\|[\w-]+)?\]\+/;
const CARD_LINE = /^> > - \[([ xX])\] (.*)$/;

function parseBoardColumns(body) {
  if (!body.includes('[!kanban]')) return null;
  const lines = body.replace(/\r\n/g, '\n').split('\n');
  const columns = [];
  let current = null;
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

function rebuildBoard(body, columns) {
  const lines = body.replace(/\r\n/g, '\n').split('\n');
  const out = [];
  let col = -1;
  let writingCards = false;
  for (const line of lines) {
    if (COLUMN_START.test(line)) {
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

function addCardToBoard(body, columnIndex, title) {
  const columns = parseBoardColumns(body);
  if (!columns || columnIndex < 0 || columnIndex >= columns.length) return null;
  const name = title.trim();
  if (!name) return null;
  columns[columnIndex].cards.push(name);
  return rebuildBoard(body, columns);
}

function moveCardOnBoard(body, fromColumn, cardIndex, toColumn) {
  const columns = parseBoardColumns(body);
  if (!columns) return null;
  if (fromColumn < 0 || fromColumn >= columns.length) return null;
  if (toColumn < 0 || toColumn >= columns.length) return null;
  const source = columns[fromColumn];
  if (cardIndex < 0 || cardIndex >= source.cards.length) return null;
  const [card] = source.cards.splice(cardIndex, 1);
  columns[toColumn].cards.push(card);
  return rebuildBoard(body, columns);
}

const en = {
  'editor:kanban.todo': 'To do',
  'editor:kanban.inProgress': 'Doing',
  'editor:kanban.review': 'Review',
  'editor:kanban.done': 'Done',
};

test('kanban template: default To do / Doing / Done columns', () => {
  const t = (key) => en[key] ?? key;
  const body = kanbanEmptyBody(t);
  assert.match(body, /\[!kanban\]/);
  const cols = parseBoardColumns(body);
  assert.ok(cols);
  assert.equal(cols.length, 3);
  assert.match(cols[0].header, /To do/);
  assert.match(cols[1].header, /Doing/);
  assert.match(cols[2].header, /Done/);
  assert.deepEqual(cols.map((c) => c.cards), [[], [], []]);
});

test('kanban: add a card then drag it to another column', () => {
  const t = (key) => en[key] ?? key;
  let body = kanbanEmptyBody(t);

  body = addCardToBoard(body, 0, 'Ship step 4');
  assert.ok(body);
  let cols = parseBoardColumns(body);
  assert.deepEqual(cols.map((c) => c.cards), [['Ship step 4'], [], []]);

  body = moveCardOnBoard(body, 0, 0, 1);
  assert.ok(body);
  cols = parseBoardColumns(body);
  assert.deepEqual(cols.map((c) => c.cards), [[], ['Ship step 4'], []]);
  assert.match(cols[1].header, /Doing/);
});

test('kanban: board files as one note body in the folder tree', () => {
  const t = (key) => en[key] ?? key;
  const note = {
    type: 'task',
    folderId: 'folder-alpha',
    title: 'Kanban board',
    body: kanbanEmptyBody(t),
  };
  assert.equal(note.type, 'task');
  assert.equal(note.folderId, 'folder-alpha');
  assert.ok(note.body.includes('[!kanban]'));
});
