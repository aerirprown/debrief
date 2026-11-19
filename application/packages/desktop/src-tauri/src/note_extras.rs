//! Game note extras: hotkey append, voice audio, reminder notifications.

use std::path::PathBuf;

use debrief_journal::{
    add_reminder, append_hotkey_line, clear_reminder, due_notifications, notification_plan,
    resolve_audio_path, write_audio, write_note_revision, Container, NewNoteRevision, NotifyKind,
    Reminder,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::device_id::install_device_id;

fn root_path(journal_root: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(journal_root);
    if !path.is_dir() {
        return Err(format!("journal root is not a directory: {journal_root}"));
    }
    Ok(path)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRevisionDto {
    pub note_id: String,
    pub game_id: String,
    pub revision_id: String,
    pub parent_revision_id: Option<String>,
    pub container_count: usize,
    pub reminder_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingNotifyDto {
    pub id: String,
    pub kind: String,
    pub game_id: String,
    pub note_id: Option<String>,
    pub container_id: Option<String>,
    pub at: String,
    pub title: String,
    pub body: String,
    pub due: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderInput {
    pub at: String,
    pub label: Option<String>,
    pub container_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteNoteInput {
    pub note_id: String,
    pub game_id: String,
    pub parent_revision_id: Option<String>,
    pub containers: Vec<ContainerDto>,
    pub reminders: Vec<ReminderDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerDto {
    pub kind: String,
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub html: Option<String>,
    pub src: Option<String>,
    pub rows: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderDto {
    pub id: String,
    pub at: String,
    pub label: Option<String>,
    pub container_id: Option<String>,
    pub cleared: bool,
}

fn kind_str(k: &NotifyKind) -> &'static str {
    match k {
        NotifyKind::Reminder => "reminder",
        NotifyKind::ComeBack => "comeBack",
    }
}

#[tauri::command]
pub fn note_append_hotkey_line(
    app: AppHandle,
    journal_root: String,
    game_id: String,
    line: String,
) -> Result<NoteRevisionDto, String> {
    let root = root_path(&journal_root)?;
    let device = install_device_id(&app)?;
    let rev = append_hotkey_line(&root, &device, &game_id, line.trim()).map_err(|e| e.to_string())?;
    Ok(NoteRevisionDto {
        note_id: rev.note_id,
        game_id: rev.game_id,
        revision_id: rev.revision_id,
        parent_revision_id: rev.parent_revision_id,
        container_count: rev.containers.len(),
        reminder_count: rev.reminders.len(),
    })
}

#[tauri::command]
pub fn note_write_audio(
    journal_root: String,
    bytes: Vec<u8>,
    extension: String,
) -> Result<String, String> {
    let root = root_path(&journal_root)?;
    let name = write_audio(&root, &bytes, &extension).map_err(|e| e.to_string())?;
    Ok(format!("audio/{name}"))
}

#[tauri::command]
pub fn note_audio_path(journal_root: String, src: String) -> Result<Option<String>, String> {
    let root = root_path(&journal_root)?;
    Ok(resolve_audio_path(&root, &src).map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub fn note_add_reminder(
    app: AppHandle,
    journal_root: String,
    note_id: String,
    input: ReminderInput,
) -> Result<NoteRevisionDto, String> {
    let root = root_path(&journal_root)?;
    let device = install_device_id(&app)?;
    let rev = add_reminder(
        &root,
        &device,
        &note_id,
        input.at,
        input.label,
        input.container_id,
    )
    .map_err(|e| e.to_string())?;
    Ok(NoteRevisionDto {
        note_id: rev.note_id,
        game_id: rev.game_id,
        revision_id: rev.revision_id,
        parent_revision_id: rev.parent_revision_id,
        container_count: rev.containers.len(),
        reminder_count: rev.reminders.len(),
    })
}

#[tauri::command]
pub fn note_clear_reminder(
    app: AppHandle,
    journal_root: String,
    note_id: String,
    reminder_id: String,
) -> Result<NoteRevisionDto, String> {
    let root = root_path(&journal_root)?;
    let device = install_device_id(&app)?;
    let rev = clear_reminder(&root, &device, &note_id, &reminder_id).map_err(|e| e.to_string())?;
    Ok(NoteRevisionDto {
        note_id: rev.note_id,
        game_id: rev.game_id,
        revision_id: rev.revision_id,
        parent_revision_id: rev.parent_revision_id,
        container_count: rev.containers.len(),
        reminder_count: rev.reminders.len(),
    })
}

#[tauri::command]
pub fn note_notification_plan(
    journal_root: String,
    now: String,
) -> Result<Vec<PendingNotifyDto>, String> {
    let root = root_path(&journal_root)?;
    let plan = notification_plan(&root, &now).map_err(|e| e.to_string())?;
    Ok(plan
        .into_iter()
        .map(|n| PendingNotifyDto {
            id: n.id,
            kind: kind_str(&n.kind).into(),
            game_id: n.game_id,
            note_id: n.note_id,
            container_id: n.container_id,
            at: n.at,
            title: n.title,
            body: n.body,
            due: n.due,
        })
        .collect())
}

#[tauri::command]
pub fn note_due_notifications(
    journal_root: String,
    now: String,
) -> Result<Vec<PendingNotifyDto>, String> {
    let root = root_path(&journal_root)?;
    let due = due_notifications(&root, &now).map_err(|e| e.to_string())?;
    Ok(due
        .into_iter()
        .map(|n| PendingNotifyDto {
            id: n.id,
            kind: kind_str(&n.kind).into(),
            game_id: n.game_id,
            note_id: n.note_id,
            container_id: n.container_id,
            at: n.at,
            title: n.title,
            body: n.body,
            due: n.due,
        })
        .collect())
}

#[tauri::command]
pub fn note_write_revision(
    app: AppHandle,
    journal_root: String,
    input: WriteNoteInput,
) -> Result<NoteRevisionDto, String> {
    let root = root_path(&journal_root)?;
    let device = install_device_id(&app)?;
    let mut containers = Vec::new();
    for c in input.containers {
        let container = match c.kind.as_str() {
            "text" => Container::Text {
                id: c.id,
                x: c.x,
                y: c.y,
                w: c.w,
                h: c.h,
                html: c.html.unwrap_or_default(),
            },
            "image" => Container::Image {
                id: c.id,
                x: c.x,
                y: c.y,
                w: c.w,
                h: c.h,
                src: c.src.unwrap_or_default(),
            },
            "table" => Container::Table {
                id: c.id,
                x: c.x,
                y: c.y,
                w: c.w,
                h: c.h,
                rows: c.rows.unwrap_or_else(|| vec![vec![String::new()]]),
            },
            "audio" => Container::Audio {
                id: c.id,
                x: c.x,
                y: c.y,
                w: c.w,
                h: c.h,
                src: c.src.unwrap_or_default(),
            },
            other => return Err(format!("unknown container kind: {other}")),
        };
        containers.push(container);
    }
    let reminders: Vec<Reminder> = input
        .reminders
        .into_iter()
        .map(|r| Reminder {
            id: r.id,
            at: r.at,
            label: r.label,
            container_id: r.container_id,
            cleared: r.cleared,
        })
        .collect();
    let rev = write_note_revision(
        &root,
        &device,
        NewNoteRevision {
            note_id: input.note_id,
            game_id: input.game_id,
            parent_revision_id: input.parent_revision_id,
            containers,
            reminders,
        },
    )
    .map_err(|e| e.to_string())?;
    Ok(NoteRevisionDto {
        note_id: rev.note_id,
        game_id: rev.game_id,
        revision_id: rev.revision_id,
        parent_revision_id: rev.parent_revision_id,
        container_count: rev.containers.len(),
        reminder_count: rev.reminders.len(),
    })
}
