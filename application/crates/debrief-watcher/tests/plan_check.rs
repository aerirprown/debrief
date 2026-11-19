//! Plan step 7 check:
//! short-lived test process → start → stop → prompt → session file.

use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use debrief_journal::{create_device_id, Journal};
use debrief_watcher::{
    finish_session, FixedFrameSampler, LocalLog, LogEvent, MockProcessSource, PendingSession,
    ProcessSource, SystemProcessSource, WatchConfig, WatchEngine, WatchEvent, WatchedGame,
};

fn tiny_png() -> Vec<u8> {
    // 1×1 PNG
    use base64_lite::decode;
    decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==")
        .unwrap()
}

mod base64_lite {
    /// Minimal base64 decode for the fixture PNG (avoids extra dep in tests).
    pub fn decode(input: &str) -> Result<Vec<u8>, ()> {
        fn val(c: u8) -> Option<u8> {
            match c {
                b'A'..=b'Z' => Some(c - b'A'),
                b'a'..=b'z' => Some(c - b'a' + 26),
                b'0'..=b'9' => Some(c - b'0' + 52),
                b'+' => Some(62),
                b'/' => Some(63),
                _ => None,
            }
        }
        let bytes: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
        let mut out = Vec::new();
        let mut i = 0;
        while i + 4 <= bytes.len() {
            let (a, b, c, d) = (bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]);
            let va = val(a).ok_or(())?;
            let vb = val(b).ok_or(())?;
            out.push((va << 2) | (vb >> 4));
            if c != b'=' {
                let vc = val(c).ok_or(())?;
                out.push(((vb & 0x0f) << 4) | (vc >> 2));
                if d != b'=' {
                    let vd = val(d).ok_or(())?;
                    out.push(((vc & 0x03) << 6) | vd);
                }
            }
            i += 4;
        }
        Ok(out)
    }
}

#[test]
fn mock_short_lived_process_prompts_and_writes_session() {
    let dir = tempfile::tempdir().unwrap();
    let device = create_device_id();
    let journal = Journal::open(dir.path().join("journal"), &device).unwrap();
    let log = LocalLog::open(dir.path().join("watcher.log")).unwrap();

    let exe = PathBuf::from(r"C:\Games\Fixture\game.exe");
    let mut engine = WatchEngine::new(
        WatchConfig {
            debounce: Duration::from_millis(50),
            grace: Duration::from_millis(50),
        },
        vec![WatchedGame {
            game_id: "game-fixture".into(),
            title: "Fixture".into(),
            match_paths: vec![exe.clone()],
        }],
    );

    let mut procs = MockProcessSource::default();
    let mut frames = FixedFrameSampler {
        png: tiny_png(),
    };
    let t0 = Instant::now();
    let wall = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);

    // Idle
    assert!(engine
        .tick(t0, wall, &mut procs, &mut frames)
        .is_empty());

    // Process appears — debounce
    procs.running = vec![exe.clone()];
    assert!(engine
        .tick(t0 + Duration::from_millis(10), wall, &mut procs, &mut frames)
        .is_empty());

    // Past debounce — session starts
    let started = engine.tick(
        t0 + Duration::from_millis(60),
        wall,
        &mut procs,
        &mut frames,
    );
    assert_eq!(started.len(), 1);
    match &started[0] {
        WatchEvent::SessionStarted { game_id, .. } => {
            assert_eq!(game_id, "game-fixture");
            log.append(&LogEvent::SessionStarted {
                game_id: game_id.clone(),
                at: "t".into(),
            })
            .unwrap();
        }
        other => panic!("expected SessionStarted, got {other:?}"),
    }

    // Process gone — grace
    procs.running.clear();
    assert!(engine
        .tick(
            t0 + Duration::from_millis(70),
            wall,
            &mut procs,
            &mut frames
        )
        .is_empty());

    // Past grace — prompt
    let ended = engine.tick(
        t0 + Duration::from_millis(130),
        wall + Duration::from_secs(120),
        &mut procs,
        &mut frames,
    );
    assert_eq!(ended.len(), 1);
    let WatchEvent::Prompt {
        game_id,
        started_at,
        stopped_at,
        frame,
        ..
    } = &ended[0]
    else {
        panic!("expected Prompt, got {:?}", ended[0]);
    };
    assert_eq!(game_id, "game-fixture");
    assert!(frame.as_ref().is_some_and(|b| b.len() > 8));
    log.append(&LogEvent::PromptOffered {
        game_id: game_id.clone(),
        at: stopped_at.clone(),
    })
    .unwrap();

    // Dismiss → session file with empty text
    let session = finish_session(
        journal.root(),
        &device,
        &PendingSession {
            game_id: game_id.clone(),
            started_at: started_at.clone(),
            stopped_at: stopped_at.clone(),
            frame: frame.clone(),
        },
        "",
        "",
    )
    .unwrap();
    assert!(session.text.is_empty());
    assert!(session.frame_id.is_some());

    let memoir = journal.read_memoir().unwrap();
    assert_eq!(memoir.sessions.len(), 1);
    assert_eq!(memoir.sessions[0].id, session.id);

    // One line submit also works
    let session2 = finish_session(
        journal.root(),
        &device,
        &PendingSession {
            game_id: game_id.clone(),
            started_at: started_at.clone(),
            stopped_at: stopped_at.clone(),
            frame: None,
        },
        "cleared the chapel",
        "side door",
    )
    .unwrap();
    assert_eq!(session2.text, "cleared the chapel");
    assert_eq!(journal.read_memoir().unwrap().sessions.len(), 2);
}

#[test]
fn real_probe_process_start_stop_prompt_and_file() {
    let probe = env!("CARGO_BIN_EXE_debrief-watcher-probe");
    let mut child: Child = Command::new(probe)
        .arg("20")
        .spawn()
        .expect("spawn probe");
    let pid = child.id();

    // Resolve the probe's absolute path from the live process list.
    let mut sys = SystemProcessSource::new();
    let mut exe: Option<PathBuf> = None;
    for _ in 0..50 {
        if let Some(proc_) = sys.list().into_iter().find(|p| p.pid == pid) {
            exe = Some(proc_.exe);
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let exe = exe.expect("probe exe path from process list");

    let dir = tempfile::tempdir().unwrap();
    let device = create_device_id();
    let journal = Journal::open(dir.path().join("journal"), &device).unwrap();

    let mut engine = WatchEngine::new(
        WatchConfig {
            debounce: Duration::from_millis(80),
            grace: Duration::from_millis(80),
        },
        vec![WatchedGame {
            game_id: "probe-game".into(),
            title: "Probe".into(),
            match_paths: vec![exe.clone()],
        }],
    );

    let mut frames = FixedFrameSampler {
        png: tiny_png(),
    };
    let t0 = Instant::now();
    let wall = SystemTime::now();

    // Wait until SessionStarted
    let mut started_at = None;
    for i in 0..40 {
        let now = t0 + Duration::from_millis(i * 40);
        for ev in engine.tick(now, wall, &mut sys, &mut frames) {
            if let WatchEvent::SessionStarted {
                started_at: s,
                game_id,
            } = ev
            {
                assert_eq!(game_id, "probe-game");
                started_at = Some(s);
            }
        }
        if started_at.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let started_at = started_at.expect("session should start while probe runs");

    // Stop the process
    let _ = child.kill();
    let _ = child.wait();

    // Wait until Prompt
    let mut prompt = None;
    let t1 = Instant::now();
    for i in 0..40 {
        let now = t1 + Duration::from_millis(i * 40);
        for ev in engine.tick(now, wall, &mut sys, &mut frames) {
            if let WatchEvent::Prompt {
                game_id,
                stopped_at,
                frame,
                ..
            } = ev
            {
                assert_eq!(game_id, "probe-game");
                prompt = Some((stopped_at, frame));
            }
        }
        if prompt.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let (stopped_at, frame) = prompt.expect("prompt after probe exits");

    let session = finish_session(
        journal.root(),
        &device,
        &PendingSession {
            game_id: "probe-game".into(),
            started_at,
            stopped_at,
            frame,
        },
        "probe line",
        "",
    )
    .unwrap();
    assert_eq!(session.text, "probe line");
    assert_eq!(journal.read_memoir().unwrap().sessions.len(), 1);

    // Ensure child is gone
    let _ = child;
}
