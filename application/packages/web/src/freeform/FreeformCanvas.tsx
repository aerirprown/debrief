/**
 * OneNote-style freeform page: click empty space for a text box, drag
 * containers, paste text / image / table.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import type { FreeformContainer, FreeformPage } from './page';
import {
  addAudio,
  addImage,
  addReminder,
  addTable,
  addTextBox,
  clearReminder,
  moveContainer,
  removeContainer,
  rowsFromClipboard,
  updateContainer,
} from './page';
import './freeform.css';

type DragState = {
  id: string;
  startX: number;
  startY: number;
  origX: number;
  origY: number;
};

type Props = {
  page: FreeformPage;
  readOnly: boolean;
  loadPicture: (src: string) => Promise<string | null>;
  storePicture: (file: File) => Promise<string | null>;
  /** Optional voice memo: store returns `audio/...` or a blob URL. */
  storeAudio?: (blob: Blob, mime: string) => Promise<string | null>;
  loadAudio?: (src: string) => Promise<string | null>;
  onPageChange: (page: FreeformPage) => void;
};

const MIN_ZOOM = 0.4;
const MAX_ZOOM = 2.5;

export function FreeformCanvas({
  page,
  readOnly,
  loadPicture,
  storePicture,
  storeAudio,
  loadAudio,
  onPageChange,
}: Props) {
  const rootRef = useRef<HTMLDivElement | null>(null);
  const surfaceRef = useRef<HTMLDivElement | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [drag, setDrag] = useState<DragState | null>(null);
  const [imageUrls, setImageUrls] = useState<Record<string, string>>({});
  const [audioUrls, setAudioUrls] = useState<Record<string, string>>({});
  const [remindAt, setRemindAt] = useState('');
  const [remindLabel, setRemindLabel] = useState('');
  const [zoom, setZoom] = useState(1);
  const pageRef = useRef(page);
  pageRef.current = page;
  const lastPoint = useRef({ x: 48, y: 48 });
  const fileAudioRef = useRef<HTMLInputElement | null>(null);
  const pinchRef = useRef<{ startDist: number; startZoom: number } | null>(null);
  const zoomRef = useRef(zoom);
  zoomRef.current = zoom;

  useEffect(() => {
    let cancelled = false;
    const needed = page.containers.filter((c) => c.kind === 'image') as Array<Extract<FreeformContainer, { kind: 'image' }>>;
    void (async () => {
      const next: Record<string, string> = {};
      for (const c of needed) {
        const url = await loadPicture(c.src);
        if (url) next[c.id] = url;
      }
      if (!cancelled) setImageUrls(next);
    })();
    return () => {
      cancelled = true;
    };
  }, [page.containers, loadPicture]);

  useEffect(() => {
    let cancelled = false;
    if (!loadAudio) {
      setAudioUrls({});
      return;
    }
    const needed = page.containers.filter((c) => c.kind === 'audio') as Array<
      Extract<FreeformContainer, { kind: 'audio' }>
    >;
    void (async () => {
      const next: Record<string, string> = {};
      for (const c of needed) {
        if (c.src.startsWith('blob:') || c.src.startsWith('data:') || c.src.startsWith('http')) {
          next[c.id] = c.src;
          continue;
        }
        const url = await loadAudio(c.src);
        if (url) next[c.id] = url;
      }
      if (!cancelled) setAudioUrls(next);
    })();
    return () => {
      cancelled = true;
    };
  }, [page.containers, loadAudio]);

  const commit = useCallback(
    (next: FreeformPage) => {
      pageRef.current = next;
      onPageChange(next);
    },
    [onPageChange],
  );

  const localPoint = (clientX: number, clientY: number) => {
    const el = surfaceRef.current;
    if (!el) return { x: 0, y: 0 };
    const rect = el.getBoundingClientRect();
    const z = zoomRef.current || 1;
    return {
      x: (clientX - rect.left) / z + el.scrollLeft,
      y: (clientY - rect.top) / z + el.scrollTop,
    };
  };

  // Phone editing: pinch-zoom the freeform surface (design: pinch-zoom and drag).
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;

    const dist = (a: Touch, b: Touch) => {
      const dx = a.clientX - b.clientX;
      const dy = a.clientY - b.clientY;
      return Math.hypot(dx, dy);
    };

    const onTouchStart = (e: TouchEvent) => {
      if (e.touches.length !== 2) {
        pinchRef.current = null;
        return;
      }
      pinchRef.current = {
        startDist: dist(e.touches[0], e.touches[1]),
        startZoom: zoomRef.current,
      };
    };
    const onTouchMove = (e: TouchEvent) => {
      const pinch = pinchRef.current;
      if (!pinch || e.touches.length !== 2) return;
      e.preventDefault();
      const next = pinch.startZoom * (dist(e.touches[0], e.touches[1]) / pinch.startDist);
      setZoom(Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, next)));
    };
    const onTouchEnd = () => {
      if (!pinchRef.current) return;
      pinchRef.current = null;
    };

    root.addEventListener('touchstart', onTouchStart, { passive: true });
    root.addEventListener('touchmove', onTouchMove, { passive: false });
    root.addEventListener('touchend', onTouchEnd);
    root.addEventListener('touchcancel', onTouchEnd);
    return () => {
      root.removeEventListener('touchstart', onTouchStart);
      root.removeEventListener('touchmove', onTouchMove);
      root.removeEventListener('touchend', onTouchEnd);
      root.removeEventListener('touchcancel', onTouchEnd);
    };
  }, []);

  const onDragMove = (e: React.PointerEvent) => {
    if (!drag) return;
    const z = zoomRef.current || 1;
    const dx = (e.clientX - drag.startX) / z;
    const dy = (e.clientY - drag.startY) / z;
    commit(moveContainer(pageRef.current, drag.id, drag.origX + dx, drag.origY + dy));
  };

  const onSurfacePointerDown = (e: React.PointerEvent) => {
    const pt = localPoint(e.clientX, e.clientY);
    lastPoint.current = pt;
    if (readOnly) return;
    if (e.target !== e.currentTarget && !(e.target as HTMLElement).classList.contains('db-freeform-surface')) return;
    if (e.button !== 0) return;
    const next = addTextBox(pageRef.current, pt.x - 8, pt.y - 8);
    const created = next.containers[next.containers.length - 1];
    commit(next);
    if (created) setSelectedId(created.id);
    e.preventDefault();
  };

  const onDragHandleDown = (e: React.PointerEvent, c: FreeformContainer) => {
    if (readOnly) return;
    e.stopPropagation();
    e.preventDefault();
    setSelectedId(c.id);
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    setDrag({ id: c.id, startX: e.clientX, startY: e.clientY, origX: c.x, origY: c.y });
  };

  const onDragUp = (e: React.PointerEvent) => {
    if (!drag) return;
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
    } catch {
      /* already released */
    }
    setDrag(null);
  };

  const onPaste = async (e: React.ClipboardEvent) => {
    if (readOnly) return;
    const pasteX = lastPoint.current.x;
    const pasteY = lastPoint.current.y;
    const items = e.clipboardData?.items;
    if (items) {
      for (const item of items) {
        if (item.type.startsWith('image/')) {
          e.preventDefault();
          const file = item.getAsFile();
          if (!file) return;
          const src = await storePicture(file);
          if (!src) return;
          const next = addImage(pageRef.current, pasteX, pasteY, src);
          const created = next.containers[next.containers.length - 1];
          commit(next);
          if (created) setSelectedId(created.id);
          return;
        }
      }
    }
    const html = e.clipboardData?.getData('text/html') ?? '';
    const text = e.clipboardData?.getData('text/plain') ?? '';
    const rows = rowsFromClipboard(html, text);
    if (rows) {
      e.preventDefault();
      const next = addTable(pageRef.current, pasteX, pasteY, rows);
      const created = next.containers[next.containers.length - 1];
      commit(next);
      if (created) setSelectedId(created.id);
      return;
    }
    if (text && !(e.target as HTMLElement).closest('[contenteditable="true"]')) {
      e.preventDefault();
      const escaped = text
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/\n/g, '<br>');
      const next = addTextBox(pageRef.current, pasteX, pasteY, `<p>${escaped}</p>`);
      const created = next.containers[next.containers.length - 1];
      commit(next);
      if (created) setSelectedId(created.id);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (readOnly || !selectedId) return;
    if (e.key === 'Delete' || e.key === 'Backspace') {
      const target = e.target as HTMLElement;
      if (target.isContentEditable || target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
      e.preventDefault();
      commit(removeContainer(pageRef.current, selectedId));
      setSelectedId(null);
    }
  };

  const openReminders = (page.reminders ?? []).filter((r) => !r.cleared);

  return (
    <div
      ref={rootRef}
      className="db-freeform-root"
      tabIndex={0}
      onPaste={onPaste}
      onKeyDown={onKeyDown}
      data-no-app-menu
      data-testid="freeform-root"
      data-zoom={zoom.toFixed(2)}
    >
      {!readOnly && (
        <div className="db-freeform-toolbar" data-testid="freeform-extras">
          {storeAudio ? (
            <>
              <button
                type="button"
                className="db-freeform-tool"
                data-testid="freeform-voice"
                onClick={() => fileAudioRef.current?.click()}
              >
                Voice memo
              </button>
              <input
                ref={fileAudioRef}
                type="file"
                accept="audio/*"
                hidden
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  e.target.value = '';
                  if (!file || !storeAudio) return;
                  void (async () => {
                    const src = await storeAudio(file, file.type || 'audio/ogg');
                    if (!src) return;
                    const next = addAudio(pageRef.current, lastPoint.current.x, lastPoint.current.y, src);
                    const created = next.containers[next.containers.length - 1];
                    commit(next);
                    if (created) setSelectedId(created.id);
                  })();
                }}
              />
            </>
          ) : null}
          <label className="db-freeform-remind">
            <span>Remind me</span>
            <input
              type="datetime-local"
              data-testid="freeform-remind-at"
              value={remindAt}
              onChange={(e) => setRemindAt(e.target.value)}
            />
            <input
              type="text"
              data-testid="freeform-remind-label"
              placeholder="Label"
              value={remindLabel}
              onChange={(e) => setRemindLabel(e.target.value)}
            />
            <button
              type="button"
              className="db-freeform-tool"
              data-testid="freeform-remind-add"
              disabled={!remindAt}
              onClick={() => {
                const iso = localDateTimeToRfc3339(remindAt);
                if (!iso) return;
                commit(
                  addReminder(
                    pageRef.current,
                    iso,
                    remindLabel.trim() || null,
                    selectedId,
                  ),
                );
                setRemindAt('');
                setRemindLabel('');
              }}
            >
              Add
            </button>
          </label>
        </div>
      )}
      {openReminders.length > 0 && (
        <ul className="db-freeform-reminders" data-testid="freeform-reminders">
          {openReminders.map((r) => (
            <li key={r.id}>
              <span>
                {r.label || 'Reminder'} · {r.at}
              </span>
              {!readOnly && (
                <button
                  type="button"
                  className="db-freeform-tool"
                  onClick={() => commit(clearReminder(pageRef.current, r.id))}
                >
                  Clear
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
      <div
        ref={surfaceRef}
        className="db-freeform-surface"
        onPointerDown={onSurfacePointerDown}
        role="presentation"
        style={{
          transform: `scale(${zoom})`,
          transformOrigin: '0 0',
          touchAction: 'pan-x pan-y',
        }}
      >
        {page.containers.map((c) => (
          <div
            key={c.id}
            className={`db-freeform-box${selectedId === c.id ? ' is-selected' : ''}${c.kind === 'text' ? ' is-text' : ''}`}
            style={{ left: c.x, top: c.y, width: c.w, height: c.h }}
            onPointerDown={(e) => {
              e.stopPropagation();
              lastPoint.current = { x: c.x, y: c.y };
              setSelectedId(c.id);
            }}
          >
            {!readOnly && (
              <div
                className="db-freeform-drag"
                onPointerDown={(e) => onDragHandleDown(e, c)}
                onPointerMove={onDragMove}
                onPointerUp={onDragUp}
                onPointerCancel={onDragUp}
                title="Drag"
              />
            )}
            {c.kind === 'text' && (
              <TextBox
                html={c.html}
                readOnly={readOnly}
                autoFocus={selectedId === c.id}
                onChange={(html) => commit(updateContainer(pageRef.current, c.id, { html }))}
              />
            )}
            {c.kind === 'image' && (
              <div className="db-freeform-image">
                {imageUrls[c.id] ? (
                  <img src={imageUrls[c.id]} alt="" draggable={false} />
                ) : (
                  <div className="db-freeform-image-empty">Image</div>
                )}
              </div>
            )}
            {c.kind === 'table' && (
              <div className="db-freeform-table-wrap">
                <table className="db-freeform-table">
                  <tbody>
                    {c.rows.map((row, ri) => (
                      <tr key={ri}>
                        {row.map((cell, ci) => (
                          <td key={ci}>{cell}</td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            {c.kind === 'audio' && (
              <div className="db-freeform-audio">
                {audioUrls[c.id] || c.src.startsWith('blob:') || c.src.startsWith('http') ? (
                  // eslint-disable-next-line jsx-a11y/media-has-caption
                  <audio controls src={audioUrls[c.id] ?? c.src} preload="metadata" />
                ) : (
                  <span className="db-freeform-image-empty">Voice memo</span>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function localDateTimeToRfc3339(local: string): string | null {
  if (!local) return null;
  const d = new Date(local);
  if (Number.isNaN(d.getTime())) return null;
  return d.toISOString();
}

function TextBox({
  html,
  readOnly,
  autoFocus,
  onChange,
}: {
  html: string;
  readOnly: boolean;
  autoFocus: boolean;
  onChange: (html: string) => void;
}) {
  const ref = useRef<HTMLDivElement | null>(null);
  const lastHtml = useRef(html);
  const seeded = useRef(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    if (!seeded.current) {
      el.innerHTML = html || '';
      lastHtml.current = html;
      seeded.current = true;
      return;
    }
    if (document.activeElement === el) return;
    if (el.innerHTML !== html) {
      el.innerHTML = html || '';
      lastHtml.current = html;
    }
  }, [html]);

  useEffect(() => {
    if (!autoFocus || readOnly) return;
    const el = ref.current;
    if (!el) return;
    el.focus();
  }, [autoFocus, readOnly]);

  return (
    <div
      ref={ref}
      className="db-freeform-text"
      contentEditable={!readOnly}
      suppressContentEditableWarning
      spellCheck
      onInput={() => {
        const el = ref.current;
        if (!el) return;
        const next = el.innerHTML;
        if (next === lastHtml.current) return;
        lastHtml.current = next;
        onChange(next);
      }}
      onPointerDown={(e) => e.stopPropagation()}
    />
  );
}

export default FreeformCanvas;
