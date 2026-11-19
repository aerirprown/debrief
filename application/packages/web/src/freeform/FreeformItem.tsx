/**
 * Freeform note in the editor pane: parse/serialize the body, flush on leave.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import type { LocalNote } from '../db';
import { stashPendingEdit } from '../flushStash';
import { pictureDataUrl, storeDrawingPicture } from '../drawing/pictures';
import { FreeformCanvas } from './FreeformCanvas';
import {
  emptyPage,
  parsePage,
  serializePage,
  type FreeformPage,
} from './page';

export function FreeformItem({
  note,
  readOnly,
  onBodyChange,
  flushRef,
}: {
  note: LocalNote;
  readOnly: boolean;
  onBodyChange: (id: string, body: string) => void;
  flushRef?: React.RefObject<(() => void) | null>;
}) {
  const id = note.id;
  const initial = parsePage(note.body) ?? emptyPage();
  const [page, setPage] = useState<FreeformPage>(initial);
  const pageRef = useRef(page);
  pageRef.current = page;
  const latestBody = useRef(serializePage(page));
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const leaving = useRef(false);

  const flush = useCallback(() => {
    if (timer.current) {
      clearTimeout(timer.current);
      timer.current = null;
    }
    const body = serializePage(pageRef.current);
    if (body === latestBody.current) return;
    latestBody.current = body;
    if (leaving.current) stashPendingEdit(id, body);
    onBodyChange(id, body);
  }, [id, onBodyChange]);

  useEffect(() => {
    if (!flushRef) return;
    flushRef.current = flush;
    return () => {
      if (flushRef.current === flush) flushRef.current = null;
    };
  }, [flush, flushRef]);

  useEffect(() => {
    return () => {
      flush();
    };
  }, [flush]);

  useEffect(() => {
    const onLeave = (e: Event) => {
      if (e.type === 'visibilitychange' && document.visibilityState === 'visible') return;
      leaving.current = true;
      try {
        flush();
      } finally {
        leaving.current = false;
      }
    };
    window.addEventListener('pagehide', onLeave);
    document.addEventListener('visibilitychange', onLeave);
    return () => {
      window.removeEventListener('pagehide', onLeave);
      document.removeEventListener('visibilitychange', onLeave);
    };
  }, [flush]);

  const onPageChange = useCallback(
    (next: FreeformPage) => {
      setPage(next);
      pageRef.current = next;
      if (timer.current) clearTimeout(timer.current);
      timer.current = setTimeout(() => {
        timer.current = null;
        const body = serializePage(pageRef.current);
        latestBody.current = body;
        onBodyChange(id, body);
      }, 280);
    },
    [id, onBodyChange],
  );

  return (
    <FreeformCanvas
      page={page}
      readOnly={readOnly}
      loadPicture={pictureDataUrl}
      storePicture={storeDrawingPicture}
      storeAudio={async (blob) => URL.createObjectURL(blob)}
      loadAudio={async (src) => src}
      onPageChange={onPageChange}
    />
  );
}
