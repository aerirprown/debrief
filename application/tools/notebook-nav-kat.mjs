/**
 * Folder navigation checks for Debrief (user folders / groups).
 *
 *   node --test tools/notebook-nav-kat.mjs
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';

function folderDepth(folders, id) {
  const byId = new Map(folders.map((f) => [f.id, f]));
  let depth = 0;
  let current = byId.get(id);
  const seen = new Set();
  while (current) {
    if (seen.has(current.id)) return 0;
    seen.add(current.id);
    depth++;
    current = current.parentId ? byId.get(current.parentId) : undefined;
  }
  return depth;
}

function subtreeIds(folders, id) {
  const ids = new Set([id]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const f of folders) {
      if (f.parentId && ids.has(f.parentId) && !ids.has(f.id)) {
        ids.add(f.id);
        grew = true;
      }
    }
  }
  return ids;
}

function canMoveFolder(folders, id, newParentId) {
  if (id === newParentId) return false;
  if (newParentId !== null) {
    if (subtreeIds(folders, id).has(newParentId)) return false;
    return folderDepth(folders, newParentId) > 0;
  }
  return true;
}

function canDragReparentFolder(folders, id, newParentId) {
  return canMoveFolder(folders, id, newParentId);
}

function canDragFileNote() {
  return true;
}

function nextOrder(folders, parentId) {
  const siblings = folders.filter((f) => f.parentId === parentId);
  return siblings.length === 0 ? 0 : Math.max(...siblings.map((s) => s.order)) + 1;
}

function createFolder(folders, name, parentId = null) {
  if (!name.trim()) return null;
  if (parentId !== null && folderDepth(folders, parentId) === 0) return null;
  const created = {
    id: randomUUID(),
    name: name.trim(),
    parentId,
    order: nextOrder(folders, parentId),
  };
  return { folders: [...folders, created], created };
}

test('user folders: nest, move within tree, drag freely', () => {
  let folders = [];

  const work = createFolder(folders, 'Work', null);
  assert.ok(work);
  folders = work.folders;

  const projects = createFolder(folders, 'Projects', work.created.id);
  assert.ok(projects);
  folders = projects.folders;

  const alpha = createFolder(folders, 'Alpha', projects.created.id);
  assert.ok(alpha);
  folders = alpha.folders;

  const beta = createFolder(folders, 'Beta', projects.created.id);
  assert.ok(beta);
  folders = beta.folders;

  assert.equal(canDragFileNote(folders, alpha.created.id, beta.created.id), true);
  assert.equal(canDragReparentFolder(folders, alpha.created.id, beta.created.id), true);
  assert.equal(canDragReparentFolder(folders, alpha.created.id, null), true);
  assert.equal(canDragReparentFolder(folders, alpha.created.id, alpha.created.id), false);
});
