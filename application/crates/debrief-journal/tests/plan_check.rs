//! Plan step 5 checks:
//! - two sessions from two device ids
//! - two edits of the same parent report a fork
//! - a note with a text box, a media id, and a reminder reads back

use std::path::PathBuf;

use debrief_journal::{
    create_device_id, days_since_stopped, list_journal_files, read_installed_games,
    return_card_for, Container, Journal, LaunchTarget, NewGame, NewNoteRevision, NewSession,
    NotifyKind, Reminder, ShelfSource,
};
use uuid::Uuid;

fn temp_journal(device_id: &str) -> (tempfile::TempDir, Journal) {
    let dir = tempfile::tempdir().expect("tempdir");
    let journal = Journal::open(dir.path(), device_id).expect("open journal");
    (dir, journal)
}

#[test]
fn two_sessions_from_two_device_ids() {
    let root = tempfile::tempdir().expect("tempdir");
    let device_a = create_device_id();
    let device_b = create_device_id();
    assert_ne!(device_a, device_b);

    let journal_a = Journal::open(root.path(), &device_a).unwrap();
    let journal_b = Journal::open(root.path(), &device_b).unwrap();

    let session_a = journal_a
        .write_session(NewSession {
            game_id: "game-elden".into(),
            started_at: "2026-11-18T01:00:00Z".into(),
            stopped_at: "2026-11-18T02:00:00Z".into(),
            text: "beat Margit".into(),
            what_next: "Stormveil".into(),
            frame_id: None,
        })
        .unwrap();

    let session_b = journal_b
        .write_session(NewSession {
            game_id: "game-elden".into(),
            started_at: "2026-11-18T03:00:00Z".into(),
            stopped_at: "2026-11-18T04:00:00Z".into(),
            text: "opened the gate".into(),
            what_next: "explore castle".into(),
            frame_id: Some("frame-1".into()),
        })
        .unwrap();

    assert_eq!(session_a.device_id, device_a);
    assert_eq!(session_b.device_id, device_b);
    assert_ne!(session_a.id, session_b.id);

    let memoir = journal_a.read_memoir().unwrap();
    assert_eq!(memoir.sessions.len(), 2);
    assert_eq!(memoir.sessions[0].id, session_a.id);
    assert_eq!(memoir.sessions[1].id, session_b.id);
    assert_eq!(memoir.sessions[0].text, "beat Margit");
    assert_eq!(memoir.sessions[1].frame_id.as_deref(), Some("frame-1"));

    // Two files on disk, named with their device ids.
    let sessions_dir = root.path().join("sessions");
    let names: Vec<_> = std::fs::read_dir(&sessions_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(names.iter().any(|n| n.contains(&device_a)));
    assert!(names.iter().any(|n| n.contains(&device_b)));
}

#[test]
fn two_edits_of_same_parent_report_a_fork() {
    let (_dir, journal) = temp_journal(&create_device_id());
    let note_id = Uuid::new_v4().to_string();
    let game_id = "game-fork".to_string();

    let root_rev = journal
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: game_id.clone(),
            parent_revision_id: None,
            containers: vec![Container::text("c1", 10.0, 10.0, 200.0, 80.0, "base")],
            reminders: vec![],
        })
        .unwrap();

    let device_b = create_device_id();
    let journal_b = Journal::open(journal.root(), &device_b).unwrap();

    let left = journal
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: game_id.clone(),
            parent_revision_id: Some(root_rev.revision_id.clone()),
            containers: vec![Container::text("c1", 10.0, 10.0, 200.0, 80.0, "left edit")],
            reminders: vec![],
        })
        .unwrap();

    let right = journal_b
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: game_id.clone(),
            parent_revision_id: Some(root_rev.revision_id.clone()),
            containers: vec![Container::text("c1", 10.0, 10.0, 200.0, 80.0, "right edit")],
            reminders: vec![],
        })
        .unwrap();

    let memoir = journal.read_memoir().unwrap();
    assert_eq!(memoir.notes.len(), 1);
    let lineage = &memoir.notes[0];
    assert_eq!(lineage.revisions.len(), 3);
    assert_eq!(lineage.forks.len(), 1);
    assert!(lineage.newest_unforked.is_none());

    let fork = &lineage.forks[0];
    assert_eq!(fork.parent_revision_id.as_deref(), Some(root_rev.revision_id.as_str()));
    assert_eq!(fork.child_revision_ids.len(), 2);
    assert!(fork.child_revision_ids.contains(&left.revision_id));
    assert!(fork.child_revision_ids.contains(&right.revision_id));
}

#[test]
fn note_with_text_media_and_reminder_reads_back() {
    let (_dir, journal) = temp_journal(&create_device_id());
    let note_id = Uuid::new_v4().to_string();
    let media_id = "img-abc";
    let reminder_id = Uuid::new_v4().to_string();
    let text_id = Uuid::new_v4().to_string();

    let written = journal
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-hades".into(),
            parent_revision_id: None,
            containers: vec![
                Container::text(&text_id, 40.0, 60.0, 280.0, 96.0, "<p>escape attempt</p>"),
                Container::image("img1", 360.0, 60.0, 240.0, 180.0, media_id),
            ],
            reminders: vec![Reminder {
                id: reminder_id.clone(),
                at: "2026-11-20T18:30:00Z".into(),
                label: Some("try again".into()),
                container_id: Some(text_id.clone()),
                cleared: false,
            }],
        })
        .unwrap();

    let memoir = journal.read_memoir().unwrap();
    assert_eq!(memoir.notes.len(), 1);
    let lineage = &memoir.notes[0];
    assert!(lineage.forks.is_empty());
    let tip = lineage.newest_unforked.as_ref().expect("single tip");
    assert_eq!(tip.revision_id, written.revision_id);
    assert_eq!(tip.containers.len(), 2);
    assert_eq!(tip.reminders.len(), 1);

    match &tip.containers[0] {
        Container::Text { id, html, .. } => {
            assert_eq!(id, &text_id);
            assert_eq!(html, "<p>escape attempt</p>");
        }
        other => panic!("expected text container, got {other:?}"),
    }
    match &tip.containers[1] {
        Container::Image { src, .. } => {
            assert_eq!(src, &format!("media/{media_id}"));
        }
        other => panic!("expected image container, got {other:?}"),
    }

    let reminder = &tip.reminders[0];
    assert_eq!(reminder.id, reminder_id);
    assert_eq!(reminder.at, "2026-11-20T18:30:00Z");
    assert_eq!(reminder.label.as_deref(), Some("try again"));
    assert_eq!(reminder.container_id.as_deref(), Some(text_id.as_str()));
    assert!(!reminder.cleared);
}

#[test]
fn return_card_has_line_frame_and_day_count() {
    let (_dir, journal) = temp_journal(&create_device_id());
    let game = journal
        .write_game_revision(NewGame {
            id: Some("game-return".into()),
            title: "Return Demo".into(),
            executables: vec![],
            cover_path: None,
            come_back_on: None,
            parent_revision_id: None,
            steam_app_id: None,
        })
        .unwrap();
    journal
        .write_session(NewSession {
            game_id: game.id.clone(),
            started_at: "2026-11-11T01:00:00Z".into(),
            stopped_at: "2026-11-11T02:30:00Z".into(),
            text: "cleared the chapel".into(),
            what_next: "open the side door".into(),
            frame_id: Some("frame-fixture-1".into()),
        })
        .unwrap();

    let card = journal
        .return_card(&game.id, "2026-11-18")
        .unwrap()
        .expect("return card");
    assert_eq!(card.line, "cleared the chapel");
    assert_eq!(card.what_next, "open the side door");
    assert_eq!(card.frame_id.as_deref(), Some("frame-fixture-1"));
    assert_eq!(card.days_since, 7);
    assert_eq!(days_since_stopped("2026-11-11T02:30:00Z", "2026-11-18"), 7);

    let games = journal.read_games().unwrap();
    assert_eq!(games.len(), 1);
    let memoir = journal.read_memoir().unwrap();
    let derived = return_card_for(&games[0], &memoir.sessions, "2026-11-18");
    assert_eq!(derived, card);
}

fn steam_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/fixtures/steam")
        .canonicalize()
        .expect("steam fixture")
}

#[test]
fn shelf_has_steam_install_and_manual_game() {
    let steam_root = steam_fixture_root();
    let steam = read_installed_games(&steam_root).unwrap();
    assert_eq!(steam.len(), 1);
    assert_eq!(steam[0].app_id, 424242);
    assert_eq!(steam[0].name, "Fixture Quest");

    let dir = tempfile::tempdir().unwrap();
    let device = create_device_id();
    let journal = Journal::open(dir.path(), &device).unwrap();

    // Manual game with a cover path and a session line.
    journal
        .write_game_revision(NewGame {
            id: Some("manual-hearth".into()),
            title: "Manual Hearth".into(),
            executables: vec![r"C:\Games\hearth.exe".into()],
            cover_path: Some("media/cover-hearth.png".into()),
            come_back_on: None,
            parent_revision_id: None,
            steam_app_id: None,
        })
        .unwrap();
    journal
        .write_session(NewSession {
            game_id: "manual-hearth".into(),
            started_at: "2026-11-10T01:00:00Z".into(),
            stopped_at: "2026-11-10T02:00:00Z".into(),
            text: "crafted a sword".into(),
            what_next: "mine ore".into(),
            frame_id: None,
        })
        .unwrap();

    let shelf = journal.shelf(Some(&steam_root), "2026-11-18").unwrap();
    assert_eq!(shelf.len(), 2, "expected one Steam + one manual: {shelf:?}");

    let steam_row = shelf.iter().find(|r| r.steam_app_id == Some(424242)).unwrap();
    assert_eq!(steam_row.title, "Fixture Quest");
    assert_eq!(steam_row.source, ShelfSource::Steam);
    assert!(matches!(
        steam_row.launch,
        LaunchTarget::Steam { app_id: 424242, .. }
    ));

    let manual = shelf.iter().find(|r| r.id == "manual-hearth").unwrap();
    assert_eq!(manual.source, ShelfSource::Manual);
    assert_eq!(manual.line, "crafted a sword");
    assert_eq!(manual.days_since, 8);
    assert_eq!(manual.cover_path.as_deref(), Some("media/cover-hearth.png"));
    assert!(matches!(manual.launch, LaunchTarget::Exe { .. }));
}

/// Plan step 9: page with reminders that fire, a voice memo, and a second
/// revision that keeps the fork rules.
#[test]
fn note_extras_reminder_voice_and_fork() {
    let root = tempfile::tempdir().unwrap();
    let device_a = create_device_id();
    let journal = Journal::open(root.path(), &device_a).unwrap();

    journal
        .write_game_revision(NewGame {
            id: Some("game-extras".into()),
            title: "Extras Quest".into(),
            executables: vec![],
            cover_path: None,
            come_back_on: Some("2026-11-10".into()),
            parent_revision_id: None,
            steam_app_id: None,
        })
        .unwrap();

    let audio_name = journal
        .write_audio(b"fake-ogg-bytes", "ogg")
        .expect("write audio");
    assert!(root.path().join("audio").join(&audio_name).is_file());

    let note_id = Uuid::new_v4().to_string();
    let reminder_id = Uuid::new_v4().to_string();
    let first = journal
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-extras".into(),
            parent_revision_id: None,
            containers: vec![
                Container::text("t1", 40.0, 40.0, 280.0, 64.0, "<p>boss timing</p>"),
                Container::audio("a1", 40.0, 120.0, 280.0, 56.0, &audio_name),
            ],
            reminders: vec![Reminder {
                id: reminder_id.clone(),
                at: "2026-11-18T12:00:00Z".into(),
                label: Some("event ends".into()),
                container_id: Some("t1".into()),
                cleared: false,
            }],
        })
        .unwrap();

    match &first.containers[1] {
        Container::Audio { src, .. } => assert_eq!(src, &format!("audio/{audio_name}")),
        other => panic!("expected audio, got {other:?}"),
    }

    // Reminders due at "now" fire (note reminder + come-back date).
    let due = journal
        .due_notifications("2026-11-18T18:00:00Z")
        .unwrap();
    assert!(
        due.iter().any(|n| n.kind == NotifyKind::Reminder && n.body == "event ends"),
        "expected note reminder to fire: {due:?}"
    );
    assert!(
        due.iter().any(|n| n.kind == NotifyKind::ComeBack && n.game_id == "game-extras"),
        "expected come-back to fire: {due:?}"
    );

    // Mid-session hotkey appends a text box as a new parent-linked revision.
    let after_hotkey = journal
        .append_hotkey_line("game-extras", "grabbed the key")
        .unwrap();
    assert_eq!(
        after_hotkey.parent_revision_id.as_deref(),
        Some(first.revision_id.as_str())
    );
    assert_eq!(after_hotkey.containers.len(), 3);
    assert!(after_hotkey.containers.iter().any(|c| matches!(
        c,
        Container::Text { html, .. } if html.contains("grabbed the key")
    )));

    // Second device edits the same parent → fork rules still hold.
    let device_b = create_device_id();
    let journal_b = Journal::open(root.path(), &device_b).unwrap();
    let left = journal
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-extras".into(),
            parent_revision_id: Some(after_hotkey.revision_id.clone()),
            containers: after_hotkey.containers.clone(),
            reminders: after_hotkey.reminders.clone(),
        })
        .unwrap();
    let right = journal_b
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-extras".into(),
            parent_revision_id: Some(after_hotkey.revision_id.clone()),
            containers: vec![Container::text("t1", 40.0, 40.0, 280.0, 64.0, "<p>other</p>")],
            reminders: vec![],
        })
        .unwrap();

    let memoir = journal.read_memoir().unwrap();
    let lineage = memoir.notes.iter().find(|n| n.note_id == note_id).unwrap();
    assert_eq!(lineage.forks.len(), 1);
    assert!(lineage.newest_unforked.is_none());
    let fork = &lineage.forks[0];
    assert!(fork.child_revision_ids.contains(&left.revision_id));
    assert!(fork.child_revision_ids.contains(&right.revision_id));
}

/// Plan step 10: two PCs, one cloud folder. Directory listing + memoir read.
/// Device ids stay outside the shared folder (per-install / beside settings).
#[test]
fn two_pcs_cloud_folder_sessions_and_forks() {
    let cloud = tempfile::tempdir().expect("cloud folder");
    let device_pc_a = create_device_id();
    let device_pc_b = create_device_id();
    assert_ne!(device_pc_a, device_pc_b);

    // Two installs open the same synced path with their own device ids.
    let pc_a = Journal::open(cloud.path(), &device_pc_a).unwrap();
    let pc_b = Journal::open(cloud.path(), &device_pc_b).unwrap();

    pc_a
        .write_game_revision(NewGame {
            id: Some("game-shared".into()),
            title: "Shared Quest".into(),
            executables: vec![],
            cover_path: None,
            come_back_on: None,
            parent_revision_id: None,
            steam_app_id: None,
        })
        .unwrap();

    let session_a = pc_a
        .write_session(NewSession {
            game_id: "game-shared".into(),
            started_at: "2026-11-18T10:00:00Z".into(),
            stopped_at: "2026-11-18T11:00:00Z".into(),
            text: "pc a cleared the bridge".into(),
            what_next: "find the key".into(),
            frame_id: None,
        })
        .unwrap();
    let session_b = pc_b
        .write_session(NewSession {
            game_id: "game-shared".into(),
            started_at: "2026-11-18T14:00:00Z".into(),
            stopped_at: "2026-11-18T15:30:00Z".into(),
            text: "pc b opened the door".into(),
            what_next: "boss room".into(),
            frame_id: None,
        })
        .unwrap();

    // Device id must not appear inside the synced folder.
    assert!(
        !cloud.path().join(".device_id").exists(),
        "device id must live beside app settings, not in the cloud folder"
    );

    // Directory listing: both session files, named with their device ids.
    let listing = list_journal_files(cloud.path()).unwrap();
    assert!(
        listing.iter().any(|p| {
            p.starts_with("sessions/") && p.contains(&session_a.id) && p.contains(&device_pc_a)
        }),
        "missing pc A session in listing: {listing:?}"
    );
    assert!(
        listing.iter().any(|p| {
            p.starts_with("sessions/") && p.contains(&session_b.id) && p.contains(&device_pc_b)
        }),
        "missing pc B session in listing: {listing:?}"
    );

    // Memoir on either PC shows both sessions.
    let memoir = pc_a.read_memoir().unwrap();
    assert_eq!(memoir.sessions.len(), 2);
    assert_eq!(memoir.sessions[0].text, "pc a cleared the bridge");
    assert_eq!(memoir.sessions[1].text, "pc b opened the door");
    assert_eq!(memoir.sessions[0].device_id, device_pc_a);
    assert_eq!(memoir.sessions[1].device_id, device_pc_b);

    // Fork: both PCs edit the same note parent; both versions remain.
    let note_id = Uuid::new_v4().to_string();
    let root_rev = pc_a
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-shared".into(),
            parent_revision_id: None,
            containers: vec![Container::text("c1", 10.0, 10.0, 200.0, 80.0, "base")],
            reminders: vec![],
        })
        .unwrap();
    let left = pc_a
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-shared".into(),
            parent_revision_id: Some(root_rev.revision_id.clone()),
            containers: vec![Container::text("c1", 10.0, 10.0, 200.0, 80.0, "from pc a")],
            reminders: vec![],
        })
        .unwrap();
    let right = pc_b
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-shared".into(),
            parent_revision_id: Some(root_rev.revision_id.clone()),
            containers: vec![Container::text("c1", 10.0, 10.0, 200.0, 80.0, "from pc b")],
            reminders: vec![],
        })
        .unwrap();

    let memoir = pc_b.read_memoir().unwrap();
    let lineage = memoir.notes.iter().find(|n| n.note_id == note_id).unwrap();
    assert_eq!(lineage.forks.len(), 1);
    assert!(lineage.newest_unforked.is_none());
    let fork = &lineage.forks[0];
    assert!(fork.child_revision_ids.contains(&left.revision_id));
    assert!(fork.child_revision_ids.contains(&right.revision_id));

    let listing = list_journal_files(cloud.path()).unwrap();
    assert!(
        listing
            .iter()
            .filter(|p| p.starts_with(&format!("notes/{note_id}/")))
            .count()
            >= 3,
        "expected root + two fork revisions on disk: {listing:?}"
    );
}

/// Plan step 11: phone path — journal R/W without watcher/Steam/launch.
/// Manual game + session, return card, note reminder → notification plan.
#[test]
fn phone_fixture_journal_without_watcher() {
    let root = tempfile::tempdir().expect("phone journal");
    let phone = create_device_id();
    let journal = Journal::open(root.path(), &phone).unwrap();

    journal
        .write_game_revision(NewGame {
            id: Some("game-phone-1".into()),
            title: "Phone Fixture".into(),
            executables: vec![],
            cover_path: None,
            come_back_on: Some("2026-12-01".into()),
            parent_revision_id: None,
            steam_app_id: None,
        })
        .unwrap();

    let session = journal
        .write_session(NewSession {
            game_id: "game-phone-1".into(),
            started_at: "2026-11-18T18:00:00Z".into(),
            stopped_at: "2026-11-18T19:15:00Z".into(),
            text: "logged from phone".into(),
            what_next: "try the side quest".into(),
            frame_id: None,
        })
        .unwrap();
    assert_eq!(session.device_id, phone);

    // Shelf with no Steam root: one manual row, no launch.
    let rows = journal.shelf(None, "2026-11-19").unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "Phone Fixture");
    assert_eq!(rows[0].line, "logged from phone");
    assert_eq!(rows[0].days_since, 1);
    assert_eq!(rows[0].source, ShelfSource::Manual);
    assert_eq!(rows[0].launch, LaunchTarget::None);

    let card = journal
        .return_card("game-phone-1", "2026-11-19")
        .unwrap()
        .expect("return card");
    assert_eq!(card.line, "logged from phone");
    assert_eq!(card.what_next, "try the side quest");
    assert_eq!(card.days_since, 1);

    let note_id = Uuid::new_v4().to_string();
    journal
        .write_note_revision(NewNoteRevision {
            note_id: note_id.clone(),
            game_id: "game-phone-1".into(),
            parent_revision_id: None,
            containers: vec![
                Container::text("t1", 20.0, 20.0, 240.0, 80.0, "boss timing"),
                Container::audio("a1", 20.0, 120.0, 200.0, 48.0, "audio/memo-1.ogg"),
            ],
            reminders: vec![Reminder {
                id: "rem-phone-1".into(),
                at: "2026-11-20T15:00:00Z".into(),
                label: Some("event ends".into()),
                container_id: Some("t1".into()),
                cleared: false,
            }],
        })
        .unwrap();

    let plan = journal
        .notification_plan("2026-11-19T12:00:00Z")
        .unwrap();
    assert!(
        plan.iter().any(|n| n.kind == NotifyKind::Reminder && n.at == "2026-11-20T15:00:00Z"),
        "expected note reminder in plan: {plan:?}"
    );
    assert!(
        plan.iter().any(|n| n.kind == NotifyKind::ComeBack),
        "expected come-back notification in plan: {plan:?}"
    );

    // No device id file inside the journal (phone install stores it in app data).
    assert!(!root.path().join(".device_id").exists());

    let listing = list_journal_files(root.path()).unwrap();
    assert!(listing.iter().any(|p| p.starts_with("sessions/") && p.contains(&session.id)));
    assert!(listing.iter().any(|p| p.starts_with(&format!("notes/{note_id}/"))));
    assert!(listing.iter().any(|p| p.starts_with("games/game-phone-1/")));
}
