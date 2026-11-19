/**
 * Freeform canvas page model checks for Debrief (Plan step 3).
 *
 *   node --test tools/freeform-kat.mjs
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

const FREEFORM_TYPE = 'debrief-freeform';

function emptyPage() {
  return {
    type: FREEFORM_TYPE,
    version: 1,
    revisionId: randomUUID(),
    parentRevisionId: null,
    containers: [],
  };
}

function serializePage(page) {
  return JSON.stringify({
    type: page.type,
    version: page.version,
    revisionId: page.revisionId,
    parentRevisionId: page.parentRevisionId,
    containers: page.containers,
  });
}

function parsePage(body) {
  if (!body.trim()) return emptyPage();
  let data;
  try {
    data = JSON.parse(body);
  } catch {
    return null;
  }
  if (!data || data.type !== FREEFORM_TYPE) return null;
  return {
    type: FREEFORM_TYPE,
    version: typeof data.version === 'number' ? data.version : 1,
    revisionId: typeof data.revisionId === 'string' ? data.revisionId : randomUUID(),
    parentRevisionId: typeof data.parentRevisionId === 'string' ? data.parentRevisionId : null,
    containers: Array.isArray(data.containers) ? data.containers : [],
  };
}

function nextRevision(page) {
  return {
    ...page,
    revisionId: randomUUID(),
    parentRevisionId: page.revisionId,
  };
}

function addTextBox(page, x, y, html = '') {
  const box = {
    id: randomUUID(),
    kind: 'text',
    x: Math.max(0, Math.round(x)),
    y: Math.max(0, Math.round(y)),
    w: 280,
    h: 96,
    html,
  };
  return { ...nextRevision(page), containers: [...page.containers, box] };
}

function addImage(page, x, y, src) {
  const box = {
    id: randomUUID(),
    kind: 'image',
    x: Math.max(0, Math.round(x)),
    y: Math.max(0, Math.round(y)),
    w: 240,
    h: 180,
    src,
  };
  return { ...nextRevision(page), containers: [...page.containers, box] };
}

function moveContainer(page, id, x, y) {
  const containers = page.containers.map((c) =>
    c.id === id ? { ...c, x: Math.max(0, Math.round(x)), y: Math.max(0, Math.round(y)) } : c,
  );
  return { ...nextRevision(page), containers };
}

function rowsFromTsv(text) {
  const trimmed = text.replace(/\r\n/g, '\n').trim();
  if (!trimmed.includes('\t') && !trimmed.includes('\n')) return null;
  const rows = trimmed.split('\n').map((line) => line.split('\t'));
  if (rows.length === 1 && rows[0].length === 1) return null;
  return rows;
}

test('freeform: two text boxes, image, move, reopen keeps layout', () => {
  let page = emptyPage();
  const birth = page.revisionId;

  page = addTextBox(page, 40, 40, '<p>alpha</p>');
  assert.equal(page.containers.length, 1);
  assert.equal(page.parentRevisionId, birth);
  const firstId = page.containers[0].id;
  const afterFirst = page.revisionId;

  page = addTextBox(page, 80, 200, '<p>beta</p>');
  assert.equal(page.containers.length, 2);
  assert.equal(page.parentRevisionId, afterFirst);

  page = addImage(page, 360, 40, 'pn:img/00000000-0000-4000-8000-000000000001');
  assert.equal(page.containers.length, 3);
  assert.equal(page.containers[2].kind, 'image');

  const moveParent = page.revisionId;
  page = moveContainer(page, firstId, 120, 60);
  assert.equal(page.parentRevisionId, moveParent);
  const moved = page.containers.find((c) => c.id === firstId);
  assert.ok(moved);
  assert.equal(moved.x, 120);
  assert.equal(moved.y, 60);

  const body = serializePage(page);
  const reopened = parsePage(body);
  assert.ok(reopened);
  assert.equal(reopened.containers.length, 3);
  assert.equal(reopened.revisionId, page.revisionId);
  assert.equal(reopened.parentRevisionId, page.parentRevisionId);
  const again = reopened.containers.find((c) => c.id === firstId);
  assert.equal(again.x, 120);
  assert.equal(again.y, 60);
  assert.equal(again.html, '<p>alpha</p>');
  assert.equal(reopened.containers.filter((c) => c.kind === 'text').length, 2);
  assert.equal(reopened.containers.filter((c) => c.kind === 'image').length, 1);
});

test('freeform: markdown body is not a freeform page', () => {
  assert.equal(parsePage('# hello\n\nworld'), null);
});

test('freeform: tsv clipboard becomes table rows', () => {
  const rows = rowsFromTsv('a\tb\nc\td');
  assert.deepEqual(rows, [
    ['a', 'b'],
    ['c', 'd'],
  ]);
});
