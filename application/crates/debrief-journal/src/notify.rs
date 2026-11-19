//! Local notification plan from note reminders and game "come back on" dates.
//! Each install schedules its own notifications from the folder; nothing pushes.

use std::path::Path;

use crate::error::Result;
use crate::game::read_games;
use crate::memoir::read_memoir;

/// Kind of local notification derived from the journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotifyKind {
    /// Note reminder date-and-time.
    Reminder,
    /// Game "come back on this day" (date only → midnight UTC of that day).
    ComeBack,
}

/// One notification the device should schedule or fire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingNotification {
    pub id: String,
    pub kind: NotifyKind,
    pub game_id: String,
    pub note_id: Option<String>,
    pub container_id: Option<String>,
    /// RFC 3339 UTC (reminders) or `YYYY-MM-DDT00:00:00Z` (come back).
    pub at: String,
    pub title: String,
    pub body: String,
    /// True when `at` is at or before `now` (should fire / overdue).
    pub due: bool,
}

/// Build the notification plan from the newest unforked tip of each note and
/// each game's `come_back_on`. Cleared reminders are omitted.
pub fn notification_plan(root: &Path, now_rfc3339: &str) -> Result<Vec<PendingNotification>> {
    let memoir = read_memoir(root)?;
    let games = read_games(root)?;
    let mut out = Vec::new();

    for game in &games {
        if let Some(day) = &game.come_back_on {
            if day.is_empty() {
                continue;
            }
            let at = format!("{day}T00:00:00Z");
            out.push(PendingNotification {
                id: format!("come-back:{}", game.id),
                kind: NotifyKind::ComeBack,
                game_id: game.id.clone(),
                note_id: None,
                container_id: None,
                at: at.clone(),
                title: game.title.clone(),
                body: format!("Come back to {}", game.title),
                due: at.as_str() <= now_rfc3339,
            });
        }
    }

    for lineage in &memoir.notes {
        let Some(tip) = &lineage.newest_unforked else {
            continue;
        };
        let game_title = games
            .iter()
            .find(|g| g.id == tip.game_id)
            .map(|g| g.title.as_str())
            .unwrap_or("Game");
        for rem in &tip.reminders {
            if rem.cleared {
                continue;
            }
            let body = rem
                .label
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Note reminder".into());
            out.push(PendingNotification {
                id: format!("reminder:{}:{}", tip.note_id, rem.id),
                kind: NotifyKind::Reminder,
                game_id: tip.game_id.clone(),
                note_id: Some(tip.note_id.clone()),
                container_id: rem.container_id.clone(),
                at: rem.at.clone(),
                title: game_title.to_string(),
                body,
                due: rem.at.as_str() <= now_rfc3339,
            });
        }
    }

    out.sort_by(|a, b| a.at.cmp(&b.at).then_with(|| a.id.cmp(&b.id)));
    Ok(out)
}

/// Reminders (and come-backs) that are due or overdue at `now`.
pub fn due_notifications(root: &Path, now_rfc3339: &str) -> Result<Vec<PendingNotification>> {
    Ok(notification_plan(root, now_rfc3339)?
        .into_iter()
        .filter(|n| n.due)
        .collect())
}
