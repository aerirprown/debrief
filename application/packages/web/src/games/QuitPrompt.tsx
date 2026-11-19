/**
 * Quit prompt window (?quit-prompt=1): one line about where you left off.
 * Enter submits; Escape dismisses (empty session text).
 */

import { useEffect, useState } from 'react';

type PromptInfo = {
  gameId: string;
  title: string;
  startedAt: string;
  stoppedAt: string;
};

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(cmd, args);
}

export function QuitPrompt() {
  const [info, setInfo] = useState<PromptInfo | null>(null);
  const [text, setText] = useState('');
  const [whatNext, setWhatNext] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const pending = await invoke<PromptInfo | null>('watcher_prompt_info');
        if (!cancelled) setInfo(pending);
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  async function submit() {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      await invoke('watcher_submit', { text: text.trim(), whatNext: whatNext.trim() });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  }

  async function dismiss() {
    if (busy) return;
    setBusy(true);
    setError(null);
    try {
      await invoke('watcher_dismiss');
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
    // Re-bind when busy flips so Escape ignores an in-flight submit.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [busy]);

  return (
    <div
      className="debrief-quit-prompt"
      data-testid="debrief-quit-prompt"
      style={{
        position: 'fixed',
        inset: 0,
        background: '#0d0d0d',
        color: '#e6e6e6',
        fontFamily: 'system-ui, sans-serif',
        padding: '1.25rem 1.5rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.75rem',
      }}
    >
      <p style={{ margin: 0, color: '#8a8a8a', fontSize: 13 }}>
        {info?.title ? `Done with ${info.title}?` : 'Where did you leave off?'}
      </p>
      <input
        autoFocus
        data-testid="quit-prompt-line"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            void submit();
          }
        }}
        placeholder="One line"
        disabled={busy}
        style={{
          background: '#1a1a1a',
          border: '1px solid #2e2e2e',
          borderRadius: 6,
          color: '#e6e6e6',
          padding: '0.6rem 0.75rem',
          fontSize: 15,
          outline: 'none',
        }}
      />
      <input
        data-testid="quit-prompt-next"
        value={whatNext}
        onChange={(e) => setWhatNext(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            void submit();
          }
        }}
        placeholder="What next (optional)"
        disabled={busy}
        style={{
          background: '#1a1a1a',
          border: '1px solid #2e2e2e',
          borderRadius: 6,
          color: '#e6e6e6',
          padding: '0.5rem 0.75rem',
          fontSize: 14,
          outline: 'none',
        }}
      />
      <div style={{ display: 'flex', gap: '0.5rem', marginTop: 'auto' }}>
        <button
          type="button"
          data-testid="quit-prompt-save"
          onClick={() => void submit()}
          disabled={busy}
          style={{
            background: '#22a05a',
            color: '#0d0d0d',
            border: 'none',
            borderRadius: 6,
            padding: '0.5rem 1rem',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          Save
        </button>
        <button
          type="button"
          data-testid="quit-prompt-dismiss"
          onClick={() => void dismiss()}
          disabled={busy}
          style={{
            background: 'transparent',
            color: '#8a8a8a',
            border: '1px solid #2e2e2e',
            borderRadius: 6,
            padding: '0.5rem 1rem',
            cursor: 'pointer',
          }}
        >
          Dismiss
        </button>
      </div>
      {error ? (
        <p style={{ margin: 0, color: '#e2c27a', fontSize: 12 }}>{error}</p>
      ) : null}
    </div>
  );
}
