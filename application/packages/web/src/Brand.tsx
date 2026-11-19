/**
 * The Debrief wordmark. This file is the ONLY place the brand markup
 * exists in the app; `check:house` fails on a hand-written copy of it.
 *
 * Forked from PrivacyNotes; product name is Debrief.
 */

// Terminal green from the Design tab (#3ddc84), used as the brand mark color.
// theme-ok: the wordmark uses one fixed accent in every palette
const BRAND_ACCENT = 'text-[#3ddc84] pn-brand-blue';

const TONE = {
  paper: 'text-[var(--brand-ink)]',
  dark: 'text-white',
  invert: 'text-[var(--wl-on-slab)]',
} as const;

/**
 * @param tone   Which ground the wordmark sits on. Defaults to paper.
 * @param suffix Unused in Debrief; kept for call-site compatibility.
 */
export function Brand({ tone = 'paper', suffix = '' }: { tone?: keyof typeof TONE; suffix?: string }) {
  return (
    <span className={`font-bold ${TONE[tone]}`}>
      <span className={BRAND_ACCENT}>Debrief</span>{suffix}
    </span>
  );
}
