import {
  phraseToSeed,
  deriveSigningKey,
  deriveEncryptionKey,
  deriveFpPepper,
  bytesToHex,
} from '@notes/shared';
import { clearLocalDatabase } from './notesRepo';
import { clearLocalSettings, loadLocalSettings, saveLocalSettings } from './userSettings';
import { DEMO_PHRASE, demoSessionIsFresh, isLocalOnlyMode, isPublicDemoMode } from './demo';
import { DEBRIEF_HIDDEN_VIEWS } from './debrief';
import { seedOnboardingNotes, SEED_MEDICATION, withSeedFactors } from './welcomeNote';
import type { AuthState } from './auth';
import type { View } from './views';

export async function buildDemoAuthState(): Promise<AuthState> {
  const seed = phraseToSeed(DEMO_PHRASE);
  const { privateKey, publicKey } = await deriveSigningKey(seed);
  const encryptionKey = deriveEncryptionKey(seed);
  const fpPepper = deriveFpPepper(seed);
  const pubkey = bytesToHex(publicKey);
  // Fresh tab session → clean slate; an in-tab refresh keeps edits.
  // Settings live in a demo-only localStorage bucket (see
  // userSettings.ts localKey) and reset alongside the notes so
  // every fresh session starts from defaults.
  // Debrief local-only never wipes (demoSessionIsFresh is false there).
  if (demoSessionIsFresh()) {
    await clearLocalDatabase();
    clearLocalSettings();
  }
  await seedOnboardingNotes(pubkey); // idempotent
  // Seed the example medication template so the wellness pill
  // renders, mirroring the first-run flow in NotesView.
  // Debrief defaults the rail to folders and hides type pillars.
  const s = loadLocalSettings();
  const medsOk = s.medications?.some((m) => m.id === SEED_MEDICATION.id);
  const factorsOk = (s.factors ?? []).length > 0;
  const browseOk = s.sidebarBrowse === 'folders';
  const viewsOk = DEBRIEF_HIDDEN_VIEWS.every((v) => (s.hiddenViews ?? []).includes(v as View));
  if (!medsOk || !factorsOk || !browseOk || !viewsOk) {
    const hidden = new Set<View>([...(s.hiddenViews ?? []), ...(DEBRIEF_HIDDEN_VIEWS as readonly View[])]);
    saveLocalSettings({
      ...s,
      medications: medsOk ? s.medications : [...(s.medications ?? []), SEED_MEDICATION],
      factors: factorsOk ? s.factors : withSeedFactors(s.factors),
      sidebarBrowse: 'folders',
      includeSubfolders: true,
      hiddenViews: [...hidden],
    });
  }
  const durableLocal = isLocalOnlyMode() && !isPublicDemoMode();
  return {
    status: 'authenticated',
    method: 'phrase',
    phrase: DEMO_PHRASE,
    pubkey,
    // The demo phrase is its own account: it never changes.
    signingPubkey: pubkey,
    encryptionKey,
    signingPrivateKey: privateKey,
    fpPepper,
    deviceId: durableLocal ? 'debrief-local' : 'demo',
    // Debrief unlocks folder create/rename without a Pro paywall.
    isPro: durableLocal ? true : false,
    isEarlySupporter: false,
    isCustodial: false,
  };
}
