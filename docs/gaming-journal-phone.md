# Gaming journal on Android and iOS

Phone builds of Debrief use the **same journal folder** as desktop: append-only
session, game, and note files. There is **no process watcher** and **no Play /
Steam launch** on phone. Log a session by hand from the Games shelf.

## Set up

1. Open **Games** → **Journal folder…**. On phone this ensures the app-data
   journal folder (`…/debrief-journal`) and remembers it as
   `debrief.journalRoot`. (Desktop still opens a system folder picker for a
   Dropbox/OneDrive/Syncthing path — see `gaming-journal-cloud.md`.)
2. To share with a PC, sync that app folder with Syncthing (or copy via Files /
   iCloud) so both installs see the same `sessions/` / `notes/` trees. A full
   SAF / security-scoped bookmark picker for arbitrary cloud trees can come
   later; the journal itself is ordinary files on disk.
3. **Add game** (title only), then **Log a session** with one line (and optional
   “what next”).
4. Open a game page for the freeform note, voice memos, and **Remind me** times.
   Come-back dates and note reminders schedule **local** notifications on this
   install only.

## What phones share with desktop

| Feature | Phone |
| --- | --- |
| Folder tree, freeform notes, kanban | Yes (pinch-zoom on freeform; Move to on kanban) |
| Shelf + return card + memoir | Yes |
| Manual session entry | Yes |
| Local reminders / come-back | Yes |
| Journal folder picker | Yes |
| Process watcher / quit prompt / hotkey | No |
| Steam scan / Play launch | No |

Device id stays in app data on that phone, never inside the synced journal.

## Check (developers)

```text
cd application/crates/debrief-journal
cargo test --test plan_check phone_fixture_journal_without_watcher -- --exact

cd application
node --test tools/phone-journal-kat.mjs
```

Android: from `application/`, with `ANDROID_HOME` set,

```text
pnpm android:build:apk
```

iOS needs a Mac (`pnpm ios:build`). The mobile invoke path is the same Rust
handler for both (`cfg(mobile)`); the plan check and fixture cover journal
behavior without a device.
