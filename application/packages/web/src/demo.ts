/**
 * Demo / local-only mode: a fully local, no-account sandbox.
 *
 * Upstream PrivacyNotes serves this at try.privacynotes.app as an
 * ephemeral demo. Debrief keeps the same no-network path permanently
 * (local-only shell): no phrase sign-up, no hosted sync. Set
 * VITE_LOCAL_ONLY=false only if you need the upstream auth/sync path.
 *
 * In demo / local-only mode the app:
 *   - uses a separate IndexedDB (DEMO_DB_NAME; durable for Debrief)
 *   - skips onboarding/auth and runs on a fixed-phrase key
 *   - makes zero Supabase/edge calls (sync + quota + settings are no-ops)
 *   - public demo: seeds sample content and treats data as ephemeral
 *   - Debrief local-only: keeps the library across sessions
 *
 * This module intentionally imports NO app modules, so db.ts can read
 * isDemoMode() at construction time without a circular import.
 */

import { APP_ORIGIN } from './hosts';

const DEMO_HOSTNAME = 'try.privacynotes.app';

/**
 * Separate IndexedDB so demo/local-only data never shares a database with
 * a networked install. Debrief's durable library uses `debrief`.
 */
export const DEMO_DB_NAME =
  import.meta.env.VITE_LOCAL_ONLY === 'false' ? 'privacynotes-demo' : 'debrief';

/** Where every "sign up / unlock" CTA in demo sends the user. */
// Demo conversions land on the app host, which boots signed-out
// visitors straight onto its unified auth card.
export const DEMO_APP_URL = APP_ORIGIN;

/**
 * Fixed BIP-39 mnemonic for the demo session. The keys derived from it
 * never protect anything - demo never encrypts to or syncs with a server
 * - so a public constant is fine. A fixed phrase keeps the derived
 * pubkey, and therefore the deterministic seed-note IDs, stable across
 * reloads so re-seeding stays idempotent.
 */
export const DEMO_PHRASE =
  'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

const DEMO_SESSION_KEY = 'pn:demo-session';

let cached: boolean | null = null;
let cachedPublicDemo: boolean | null = null;
let cachedLocalOnly: boolean | null = null;

/**
 * Debrief default: local library only. Upstream auth/sync stays available
 * when VITE_LOCAL_ONLY=false.
 */
export function isLocalOnlyMode(): boolean {
  if (cachedLocalOnly !== null) return cachedLocalOnly;
  cachedLocalOnly = import.meta.env.VITE_LOCAL_ONLY !== 'false';
  return cachedLocalOnly;
}

/** True for the public ephemeral demo (hostname / ?demo=1 / VITE_DEMO_MODE). */
export function isPublicDemoMode(): boolean {
  if (cachedPublicDemo !== null) return cachedPublicDemo;
  if (typeof window === 'undefined') {
    cachedPublicDemo = import.meta.env.VITE_DEMO_MODE === 'true';
    return cachedPublicDemo;
  }
  try {
    const host = window.location.hostname;
    const params = new URLSearchParams(window.location.search);
    cachedPublicDemo =
      host === DEMO_HOSTNAME ||
      params.get('demo') === '1' ||
      import.meta.env.VITE_DEMO_MODE === 'true';
  } catch {
    cachedPublicDemo = false;
  }
  return cachedPublicDemo;
}

/** True when the app skips hosted auth/sync (public demo or Debrief local-only). */
export function isDemoMode(): boolean {
  if (cached !== null) return cached;
  cached = isPublicDemoMode() || isLocalOnlyMode();
  return cached;
}

/**
 * Bucket a CREDENTIAL storage key per mode, the way DEMO_DB_NAME
 * buckets the database and settingsLocalKey buckets settings. The
 * demo's Security tabs are deliberately usable (Pro teaser), and
 * `?demo=1` is valid on ANY origin - so unbucketed, those tabs would
 * write the SAME localStorage keys as a real signed-in account on
 * that origin, and a demo session could overwrite the real user's
 * PIN-wrapped phrase with the demo phrase. Every
 * phrase, PIN and biometric key definition goes through here, so a
 * demo session reads and writes only its own `.demo` bucket and a
 * real account's credentials are untouchable from demo. This module
 * imports nothing, so any credential module can import it cycle-free.
 */
export function credentialKey(base: string): string {
  return isDemoMode() ? `${base}.demo` : base;
}

/**
 * True when a Pro-gated feature should behave as unlocked: a real Pro
 * account, or the public demo, which unlocks everything client-side so
 * visitors can try the whole app before signing up.
 *
 * This drives BEHAVIOUR only. The gold rocket / "Pro" badges still key
 * off `isPro`, so a demo visitor uses the feature and still sees it
 * labelled as Pro - that is the whole point of the teaser.
 *
 * A server-backed feature needs more than this flag. Note history
 * lives in Supabase `note_versions` behind an `is_pro()` RLS policy
 * and demo makes zero server calls, so noteVersions.ts pairs this
 * gate with a local, demo-only snapshot store rather than reaching
 * the server.
 */
export function proUnlocked(isPro: boolean | null | undefined): boolean {
  return (isPro ?? false) || isDemoMode() || isLocalOnlyMode();
}

/**
 * Pre-render cleanup for a FRESH demo tab session: drop every
 * demo-bucketed credential (the `.demo`-suffixed keys credentialKey
 * mints - phrase, PIN cache, wrap blobs, lockout, unlocked marker).
 * Without this, a demo visitor who set a PIN and app lock leaves the
 * NEXT visitor on this browser a locked demo with an unknown PIN.
 * Called from main.tsx BEFORE React mounts, because App.tsx computes
 * its locked state synchronously at first render - an async wipe would
 * strand a fresh session behind a lock whose blob was just deleted.
 * Peeks at the session marker without consuming it; the DB wipe in
 * demoAuth.ts still sees the session as fresh afterwards. The demo
 * SETTINGS bucket survives on purpose (existing behavior - an in-tab
 * refresh keeps preferences); appLockEnabled without a blob is inert.
 */
/**
 * True for a key the demo owns. Credential keys are bucketed with a
 * `.demo` suffix by credentialKey, which is what keeps a `?demo=1`
 * session on a real install off the real account's phrase envelope,
 * PIN wrap and biometric wrap. Anything that sweeps storage in demo
 * matches on this rather than on the `privacynotes.` prefix, because
 * the prefix is exactly what the two modes share.
 */
export function isDemoOwnedKey(key: string): boolean {
  return key.endsWith('.demo');
}

export function clearFreshDemoCredentials(): void {
  if (!isDemoMode()) return;
  try {
    if (sessionStorage.getItem(DEMO_SESSION_KEY)) return;
  } catch {
    return;
  }
  for (const store of [localStorage, sessionStorage]) {
    try {
      for (let i = store.length - 1; i >= 0; i--) {
        const k = store.key(i);
        if (k && isDemoOwnedKey(k)) store.removeItem(k);
      }
    } catch {
      /* ignore */
    }
  }
}

/**
 * Returns true the first time it's called in a tab session, false on
 * every subsequent call - including after an in-tab refresh, because the
 * marker lives in sessionStorage. Used to decide whether to wipe and
 * reseed the demo DB: fresh tab session = clean slate, refresh = keep the
 * user's edits. Closing the tab clears sessionStorage, so the next visit
 * starts fresh.
 */
export function demoSessionIsFresh(): boolean {
  // Debrief local-only keeps the library; only the public demo wipes.
  if (isLocalOnlyMode() && !isPublicDemoMode()) return false;
  try {
    if (sessionStorage.getItem(DEMO_SESSION_KEY)) return false;
    sessionStorage.setItem(DEMO_SESSION_KEY, '1');
    return true;
  } catch {
    return true;
  }
}
