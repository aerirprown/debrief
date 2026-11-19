import { useCallback, useEffect, useState } from 'react';
import { isPhone } from '../autoBackup/destination';
import {
  addManualGame,
  addManualSession,
  defaultSteamRoot,
  ensureJournal,
  launchShelfGame,
  listShelf,
  pickJournalFolder,
  todayIsoDay,
  type ShelfRow,
} from './shelfApi';
import './gamesShelf.css';

type Props = {
  /** Absolute journal folder path (or fixture path in tests). */
  journalRoot: string;
  /** Optional Steam library root; null = auto-detect. */
  steamRoot?: string | null;
  asOfDay?: string;
  /** When set, skip Tauri and show these rows (KAT / storybook). */
  rows?: ShelfRow[] | null;
  onClose?: () => void;
  /** Persist a user-picked journal folder (cloud path on both PCs / phones). */
  onJournalRootChange?: (path: string) => void;
  /** Force phone UI (tests); otherwise uses isPhone(). */
  phone?: boolean;
};

function daysLabel(n: number): string {
  if (n <= 0) return 'Today';
  if (n === 1) return '1 day ago';
  return `${n} days ago`;
}

/**
 * Games shelf: Steam installs + manual games, last line, days since, launch.
 * On phone: no Steam, no Play, manual session log, same journal folder picker.
 */
export function GamesShelf({
  journalRoot,
  steamRoot,
  asOfDay,
  rows: rowsProp,
  onClose,
  onJournalRootChange,
  phone: phoneProp,
}: Props) {
  const phone = phoneProp ?? isPhone();
  const [rows, setRows] = useState<ShelfRow[]>(rowsProp ?? []);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [manualTitle, setManualTitle] = useState('');
  const [manualExe, setManualExe] = useState('');
  const [resolvedSteam, setResolvedSteam] = useState<string | null>(steamRoot ?? null);
  const [logGameId, setLogGameId] = useState('');
  const [logText, setLogText] = useState('');
  const [logNext, setLogNext] = useState('');

  const reload = useCallback(async () => {
    if (rowsProp) {
      setRows(rowsProp);
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await ensureJournal(journalRoot);
      let steam: string | null = null;
      if (!phone) {
        steam = steamRoot ?? resolvedSteam;
        if (steam === undefined || steam === null) {
          steam = await defaultSteamRoot();
          setResolvedSteam(steam);
        }
      }
      const list = await listShelf({
        journalRoot,
        steamRoot: steam,
        asOfDay: asOfDay ?? todayIsoDay(),
        phone,
      });
      setRows(list);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [journalRoot, steamRoot, resolvedSteam, asOfDay, rowsProp, phone]);

  useEffect(() => {
    void reload();
  }, [reload]);

  useEffect(() => {
    if (!logGameId && rows.length) setLogGameId(rows[0].id);
  }, [rows, logGameId]);

  async function onLaunch(row: ShelfRow) {
    if (!row.canLaunch || rowsProp || phone) return;
    try {
      await launchShelfGame(row);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function onAddManual() {
    const title = manualTitle.trim();
    if (!title || rowsProp) return;
    setBusy(true);
    setError(null);
    try {
      await ensureJournal(journalRoot);
      await addManualGame({
        journalRoot,
        title,
        executable: phone ? undefined : (manualExe.trim() || undefined),
      });
      setManualTitle('');
      setManualExe('');
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  }

  async function onLogSession() {
    const text = logText.trim();
    const gameId = logGameId || rows[0]?.id;
    if (!text || !gameId || rowsProp) return;
    setBusy(true);
    setError(null);
    try {
      await ensureJournal(journalRoot);
      await addManualSession({
        journalRoot,
        gameId,
        text,
        whatNext: logNext.trim(),
      });
      setLogText('');
      setLogNext('');
      await reload();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  }

  async function onPickJournal() {
    if (rowsProp) return;
    setBusy(true);
    setError(null);
    try {
      const picked = await pickJournalFolder();
      if (picked) {
        await ensureJournal(picked);
        onJournalRootChange?.(picked);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div
      className="debrief-shelf"
      data-testid="debrief-games-shelf"
      data-phone={phone ? '1' : '0'}
    >
      <header className="debrief-shelf__header">
        <h2 className="debrief-shelf__title">Games</h2>
        {onClose ? (
          <button type="button" className="debrief-shelf__close" onClick={onClose}>
            Close
          </button>
        ) : null}
      </header>

      <section className="debrief-shelf__journal" aria-label="Journal folder">
        <div className="debrief-shelf__journal-path" data-testid="journal-root-path" title={journalRoot}>
          {journalRoot || 'No journal folder'}
        </div>
        <button
          type="button"
          className="debrief-shelf__journal-pick"
          data-testid="journal-pick-folder"
          disabled={busy || !!rowsProp}
          onClick={() => void onPickJournal()}
        >
          Journal folder…
        </button>
        <p className="debrief-shelf__cloud-help" data-testid="journal-cloud-help">
          {phone
            ? 'Journal folder… uses this phone’s app storage (or a path you already set). Sync that folder with Syncthing / Files / iCloud the same way as on PC. No game watcher — log a session by hand.'
            : 'Point this at a folder your sync client already watches (Dropbox, OneDrive, Syncthing, …). Use the same path on each PC. Device id stays on this install, not in the folder.'}
        </p>
      </section>

      <ul className="debrief-shelf__list">
        {rows.map((row) => (
          <li key={row.id} className="debrief-shelf__row" data-testid="shelf-row" data-source={row.source}>
            <div className="debrief-shelf__cover" aria-hidden>
              {row.coverPath ? '▣' : '□'}
            </div>
            <div className="debrief-shelf__meta">
              <div className="debrief-shelf__name">{row.title}</div>
              <div className="debrief-shelf__line" data-testid="shelf-line">
                {row.line || '—'}
              </div>
              <div className="debrief-shelf__days" data-testid="shelf-days">
                {daysLabel(row.daysSince)}
                <span className="debrief-shelf__source"> · {row.source}</span>
              </div>
            </div>
            {!phone && row.canLaunch ? (
              <button
                type="button"
                className="debrief-shelf__launch"
                data-testid="shelf-launch"
                disabled={busy || !!rowsProp}
                onClick={() => void onLaunch(row)}
              >
                Play
              </button>
            ) : null}
          </li>
        ))}
        {!rows.length && !busy ? (
          <li className="debrief-shelf__empty">
            {phone
              ? 'No games yet. Add a manual game, then log a session.'
              : 'No games yet. Add a manual game or point at Steam.'}
          </li>
        ) : null}
      </ul>

      <section className="debrief-shelf__add" aria-label="Add manual game">
        <input
          value={manualTitle}
          onChange={(e) => setManualTitle(e.target.value)}
          placeholder="Manual game title"
          data-testid="shelf-manual-title"
          disabled={busy || !!rowsProp}
        />
        {!phone ? (
          <input
            value={manualExe}
            onChange={(e) => setManualExe(e.target.value)}
            placeholder="Executable path (optional)"
            data-testid="shelf-manual-exe"
            disabled={busy || !!rowsProp}
          />
        ) : null}
        <button
          type="button"
          data-testid="shelf-manual-add"
          disabled={busy || !!rowsProp || !manualTitle.trim()}
          onClick={() => void onAddManual()}
        >
          Add game
        </button>
      </section>

      <section className="debrief-shelf__log" aria-label="Log a session" data-testid="shelf-log-session">
        <div className="debrief-shelf__log-title">Log a session</div>
        <select
          value={logGameId}
          onChange={(e) => setLogGameId(e.target.value)}
          data-testid="shelf-log-game"
          disabled={busy || !!rowsProp || !rows.length}
        >
          {!rows.length ? <option value="">Add a game first</option> : null}
          {rows.map((r) => (
            <option key={r.id} value={r.id}>
              {r.title}
            </option>
          ))}
        </select>
        <input
          value={logText}
          onChange={(e) => setLogText(e.target.value)}
          placeholder="What happened (one line)"
          data-testid="shelf-log-text"
          disabled={busy || !!rowsProp || !rows.length}
        />
        <input
          value={logNext}
          onChange={(e) => setLogNext(e.target.value)}
          placeholder="What next (optional)"
          data-testid="shelf-log-next"
          disabled={busy || !!rowsProp || !rows.length}
        />
        <button
          type="button"
          data-testid="shelf-log-submit"
          disabled={busy || !!rowsProp || !rows.length || !logText.trim()}
          onClick={() => void onLogSession()}
        >
          Save session
        </button>
      </section>

      {error ? <p className="debrief-shelf__error">{error}</p> : null}
      {busy ? <p className="debrief-shelf__busy">Loading…</p> : null}
    </div>
  );
}
