/**
 * Mid-session hotkey window (?mid-session=1): one line → text box on the game page.
 * Enter submits; Escape dismisses without writing.
 */

import { useEffect, useState } from 'react';

type Ctx = {
  journalRoot: string;
  gameId: string;
  title: string;
};

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

export function MidSessionPrompt() {
  const [ctx, setCtx] = useState<Ctx | null>(null);
  const [text, setText] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const pending = await invoke<Ctx | null>('mid_session_context');
        if (!cancelled) setCtx(pending);
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  async function submit() {
    if (busy || !ctx) return;
    const line = text.trim();
    if (!line) return;
    setBusy(true);
    setError(null);
    try {
      await invoke('mid_session_submit', {
        journalRoot: ctx.journalRoot,
        gameId: ctx.gameId,
        line,
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  }

  async function dismiss() {
    if (busy) return;
    setBusy(true);
    try {
      await invoke('mid_session_dismiss');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  }

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        void dismiss();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [busy]);

  return (
    <div
      className="debrief-mid-session"
      data-testid="debrief-mid-session"
      style={{
        position: 'fixed',
        inset: 0,
        background: '#0d0d0d',
        color: '#e6e6e6',
        fontFamily: 'system-ui, sans-serif',
        padding: '1rem 1.25rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.65rem',
      }}
    >
      <p style={{ margin: 0, color: '#8a8a8a', fontSize: 13 }}>
        {ctx?.title ? `Note — ${ctx.title}` : 'Quick note'}
      </p>
      <input
        autoFocus
        data-testid="mid-session-line"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            void submit();
          }
        }}
        placeholder="One line"
        disabled={busy || !ctx}
        style={{
          background: '#1a1a1a',
          border: '1px solid #2e2e2e',
          borderRadius: 6,
          color: '#e6e6e6',
          padding: '0.55rem 0.75rem',
          fontSize: 15,
          outline: 'none',
        }}
      />
      <div style={{ display: 'flex', gap: '0.5rem' }}>
        <button
          type="button"
          data-testid="mid-session-save"
          onClick={() => void submit()}
          disabled={busy || !ctx || !text.trim()}
          style={{
            background: '#22a05a',
            color: '#0d0d0d',
            border: 'none',
            borderRadius: 6,
            padding: '0.45rem 0.9rem',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          Add
        </button>
        <button
          type="button"
          data-testid="mid-session-dismiss"
          onClick={() => void dismiss()}
          disabled={busy}
          style={{
            background: 'transparent',
            color: '#8a8a8a',
            border: '1px solid #2e2e2e',
            borderRadius: 6,
            padding: '0.45rem 0.9rem',
            cursor: 'pointer',
          }}
        >
          Cancel
        </button>
      </div>
      {error ? (
        <p style={{ margin: 0, color: '#e2c27a', fontSize: 12 }}>{error}</p>
      ) : null}
    </div>
  );
}
