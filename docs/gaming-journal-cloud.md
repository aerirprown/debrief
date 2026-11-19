# Gaming journal in a cloud folder

Debrief does not sync the gaming journal for you. Point the journal at a folder
your own sync client already watches (Dropbox, OneDrive, Google Drive,
Syncthing, iCloud Drive, and so on). The app never signs in to those services.

## On each PC

1. Create a folder inside your sync client, for example  
   `Dropbox/Debrief Journal` or `OneDrive/Debrief/journal`.
2. In Debrief, open **Games** and choose **Journal folder…**, then pick that
   folder (or set the same path on both machines).
3. Play and quit as usual. Each save is a **new file** under that folder.

Do the same on the second PC, pointing at the **same synced path**. After the
client finishes copying, open Debrief on either machine: the memoir and shelf
show sessions from both installs.

Phones can point at the same folder (Files / SAF, iCloud Drive, Syncthing).
See `gaming-journal-phone.md` — no watcher on phone; log sessions by hand.

## What syncs

```text
journal/
  sessions/{sessionId}_{deviceId}.json
  notes/{noteId}/{revisionId}_{deviceId}.json
  games/{gameId}/{revisionId}_{deviceId}.json
  frames/  media/  audio/
```

Every write includes a random id and this install’s device id, so two PCs can
add a session while offline without overwriting each other. When both come
back online, both files remain; the memoir lists both.

If two edits of the same note name the same parent, that is a **fork**. Both
versions stay on disk and in the memoir until you keep one (or keep both).

## What must not live in the cloud folder

| File | Where it lives |
| --- | --- |
| Device id | App data on this install (`device_id`), not in the journal |
| Watcher log | App data on this PC (`debrief-watcher.log`) |

Do not put a `.git` directory inside the cloud folder. A Git remote as a
separate sync path may come later; Dropbox copying `.git` can corrupt it.

## Check (developers)

```text
cd application/crates/debrief-journal
cargo test --test plan_check two_pcs_cloud_folder_sessions_and_forks -- --exact

cd application
node --test tools/cloud-journal-kat.mjs
```

The plan check opens one shared temp directory as two journals with two device
ids, writes a session from each, lists the directory, reads the memoir (both
sessions), then creates a note fork (both versions).
