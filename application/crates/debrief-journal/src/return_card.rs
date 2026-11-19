//! Derived return card for a game: last line, what next, days since, frame.

use crate::game::GameRevision;
use crate::session::Session;

/// What the return card shows when opening a game page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnCard {
    pub game_id: String,
    pub title: String,
    pub line: String,
    pub what_next: String,
    pub days_since: u32,
    pub frame_id: Option<String>,
    pub session_id: Option<String>,
    pub stopped_at: Option<String>,
}

/// Newest session for `game_id` by `stopped_at` (then `started_at`, then id).
pub fn newest_session<'a>(sessions: &'a [Session], game_id: &str) -> Option<&'a Session> {
    sessions
        .iter()
        .filter(|s| s.game_id == game_id)
        .max_by(|a, b| {
            a.stopped_at
                .cmp(&b.stopped_at)
                .then_with(|| a.started_at.cmp(&b.started_at))
                .then_with(|| a.id.cmp(&b.id))
        })
}

/// Whole UTC calendar days between `stopped_at` (RFC 3339) and `as_of_day` (YYYY-MM-DD).
/// Returns 0 when the stop day is today or in the future relative to `as_of_day`.
pub fn days_since_stopped(stopped_at: &str, as_of_day: &str) -> u32 {
    let stop_day = &stopped_at[..stopped_at.find('T').unwrap_or(stopped_at.len())];
    let stop = parse_ymd(stop_day);
    let as_of = parse_ymd(as_of_day);
    match (stop, as_of) {
        (Some(s), Some(a)) if a > s => (a - s) as u32,
        _ => 0,
    }
}

fn parse_ymd(s: &str) -> Option<i64> {
    let mut parts = s.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    // Civil day number (Howard Hinnant). Good enough for day counts.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mpb = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mpb + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

pub fn return_card_for(
    game: &GameRevision,
    sessions: &[Session],
    as_of_day: &str,
) -> ReturnCard {
    let session = newest_session(sessions, &game.id);
    let (line, what_next, frame_id, session_id, stopped_at, days_since) = match session {
        Some(s) => (
            s.text.clone(),
            s.what_next.clone(),
            s.frame_id.clone(),
            Some(s.id.clone()),
            Some(s.stopped_at.clone()),
            days_since_stopped(&s.stopped_at, as_of_day),
        ),
        None => (String::new(), String::new(), None, None, None, 0),
    };
    ReturnCard {
        game_id: game.id.clone(),
        title: game.title.clone(),
        line,
        what_next,
        days_since,
        frame_id,
        session_id,
        stopped_at,
    }
}
