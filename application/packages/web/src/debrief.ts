/**
 * Debrief product flags for this PrivacyNotes fork.
 * Always-on product choices — not runtime env toggles.
 */

/** Global Ninja / Jump-to launcher is removed in Debrief. */
export const DEBRIEF_NINJA = false;

/**
 * Type pillars (Notes, Files, Vault, …) are hidden from the sidebar.
 * Primary nav is user folders; open a folder on All Items so mixed types show.
 */
export const DEBRIEF_HIDDEN_VIEWS = [
  'dashboard',
  'all',
  'tasks',
  'drawings',
  'vault',
  'files',
  'journal',
  'calendar',
  'bookmarks',
  'contacts',
  'markdown',
] as const;
