#![allow(clippy::module_name_repetitions)]
//! Centralized export progress state and hooks.
//!
//! This module provides a reactive context for tracking export progress across the app,
//! with a cross-thread safe reporting API you can pass into `spawn_blocking` tasks.
//!
//! Usage pattern:
//! - Wrap your app (or a subtree that needs export progress UI) with `ExportProgressProvider`
//! - In UI code, call `let ctx = use_export_progress();`
//! - Start a task and obtain a reporter: `let reporter = ctx.start_task(kind, format, suggested_filename);`
//! - Pass `reporter.as_callback()` into `ExportOptions::progress_callback`
//! - Or call `reporter.update(..)`, `reporter.complete(..)`, `reporter.fail(..)` from background threads
//!
//! Notes:
//! - UI signals are mutated only on the UI thread by an async loop inside the provider, fed by a Tokio mpsc channel.
//! - The reporter is `Send + Sync` and can be freely moved into `tokio::task::spawn_blocking`.

use dioxus::prelude::*;
use uuid::Uuid;

use chrono::{DateTime, Utc};
use std::path::PathBuf;
use tokio::sync::mpsc::unbounded_channel as mpsc_unbounded_channel;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Kind of export being performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportKind {
    Course(Uuid),
    Plan(Uuid),
    Notes(Uuid),
    AllData,
    Other(String),
}

impl ExportKind {
    pub fn label(&self) -> String {
        match self {
            ExportKind::Course(id) => format!("Course ({id})"),
            ExportKind::Plan(id) => format!("Plan ({id})"),
            ExportKind::Notes(id) => format!("Notes ({id})"),
            ExportKind::AllData => "All Data".to_string(),
            ExportKind::Other(s) => s.clone(),
        }
    }
}

/// Current status of an export task
#[derive(Debug, Clone, PartialEq)]
pub enum ExportTaskStatus {
    Running,
    Completed { saved_path: Option<PathBuf> },
    Failed { error: String },
}

/// Export task with metadata and progress
#[derive(Debug, Clone)]
pub struct ExportTask {
    pub id: Uuid,
    pub kind: ExportKind,
    pub format: crate::export::ExportFormat,
    pub suggested_filename: Option<String>,

    pub percentage: f32, // 0.0..=100.0
    pub message: String,

    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: ExportTaskStatus,
}

impl ExportTask {
    fn new(
        kind: ExportKind,
        format: crate::export::ExportFormat,
        suggested_filename: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            kind,
            format,
            suggested_filename,
            percentage: 0.0,
            message: "Queued".to_string(),
            started_at: now,
            updated_at: now,
            status: ExportTaskStatus::Running,
        }
    }
}

/// Progress events delivered from background tasks into the UI
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    Start(ExportTask),
    Update {
        id: Uuid,
        percentage: f32,
        message: String,
    },
    Complete {
        id: Uuid,
        saved_path: Option<PathBuf>,
    },
    Fail {
        id: Uuid,
        error: String,
    },
    ClearAll,
}

/// Context for centralized export progress tracking
#[derive(Clone)]
pub struct ExportProgressContext {
    pub tasks: Signal<Vec<ExportTask>>,
    sender: UnboundedSender<ProgressEvent>,
}

impl ExportProgressContext {
    /// Start tracking a new export task on the UI thread and return a reporter
    pub fn start_task(
        &mut self,
        kind: ExportKind,
        format: crate::export::ExportFormat,
        suggested_filename: Option<String>,
    ) -> ExportProgressReporter {
        let task = ExportTask::new(kind, format, suggested_filename);
        // Push immediately (UI thread)
        {
            let mut tasks = self.tasks.read().clone();
            tasks.push(task.clone());
            self.tasks.set(tasks);
        }
        // Also notify the event loop to normalize state flow
        let _ = self.sender.send(ProgressEvent::Start(task.clone()));
        ExportProgressReporter::new(task.id, self.sender.clone())
    }

    /// Produce a boxed callback suitable for `ExportOptions::progress_callback`
    /// All invocations will go through the cross-thread channel and update UI state.
    pub fn callback_for(&self, id: Uuid) -> Box<dyn Fn(f32, String) + Send + Sync + 'static> {
        let sender = self.sender.clone();
        Box::new(move |pct: f32, msg: String| {
            let _ = sender.send(ProgressEvent::Update {
                id,
                percentage: pct.clamp(0.0, 100.0),
                message: msg,
            });
        })
    }

    /// Clear all finished (Completed/Failed) tasks from the list
    pub fn clear_finished(&mut self) {
        let tasks = self.tasks.read();
        let remaining: Vec<ExportTask> = tasks
            .iter()
            .cloned()
            .filter(|t| matches!(t.status, ExportTaskStatus::Running))
            .collect();
        drop(tasks);
        self.tasks.set(remaining);
    }

    /// Clear everything (including running tasks)
    pub fn clear_all(&mut self) {
        self.tasks.set(Vec::new());
        let _ = self.sender.send(ProgressEvent::ClearAll);
    }

    /// Get a single task by id
    pub fn get_task(&self, id: Uuid) -> Option<ExportTask> {
        self.tasks.read().iter().find(|t| t.id == id).cloned()
    }

    /// Convenience helper to create a reporter directly without starting the task in UI.
    /// Primarily useful when the caller wants to initialize and start from a background task.
    pub fn reporter_for(&self, id: Uuid) -> ExportProgressReporter {
        ExportProgressReporter::new(id, self.sender.clone())
    }
}

/// A cross-thread safe progress reporter handle
#[derive(Clone)]
pub struct ExportProgressReporter {
    id: Uuid,
    sender: UnboundedSender<ProgressEvent>,
}

impl ExportProgressReporter {
    fn new(id: Uuid, sender: UnboundedSender<ProgressEvent>) -> Self {
        Self { id, sender }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Update the task progress
    pub fn update(&self, percentage: f32, message: impl Into<String>) {
        let _ = self.sender.send(ProgressEvent::Update {
            id: self.id,
            percentage: percentage.clamp(0.0, 100.0),
            message: message.into(),
        });
    }

    /// Mark the task as completed (optionally with a saved file path)
    pub fn complete(&self, saved_path: Option<PathBuf>) {
        let _ = self.sender.send(ProgressEvent::Complete {
            id: self.id,
            saved_path,
        });
    }

    /// Mark the task as failed with an error message
    pub fn fail(&self, error: impl Into<String>) {
        let _ = self.sender.send(ProgressEvent::Fail {
            id: self.id,
            error: error.into(),
        });
    }

    /// Get a boxed closure compatible with `ExportOptions::progress_callback`
    pub fn as_callback(&self) -> Box<dyn Fn(f32, String) + Send + Sync + 'static> {
        let id = self.id;
        let sender = self.sender.clone();
        Box::new(move |pct: f32, msg: String| {
            let _ = sender.send(ProgressEvent::Update {
                id,
                percentage: pct.clamp(0.0, 100.0),
                message: msg,
            });
        })
    }
}

/// Export progress provider: owns the event loop and exposes the context
#[component]
pub fn ExportProgressProvider(children: Element) -> Element {
    // Reactive task list
    let mut tasks = use_signal(|| Vec::<ExportTask>::new());

    // Cross-thread event channel
    let (tx, mut rx): (
        UnboundedSender<ProgressEvent>,
        UnboundedReceiver<ProgressEvent>,
    ) = mpsc_unbounded_channel();

    // Install context
    let ctx = ExportProgressContext {
        tasks: tasks.clone(),
        sender: tx.clone(),
    };
    use_context_provider(|| ctx);

    // Async event loop to safely mutate UI signals from mpsc channel
    spawn(async move {
        while let Some(evt) = rx.recv().await {
            match evt {
                ProgressEvent::Start(task) => {
                    let mut list = tasks.read().clone();
                    // Avoid duplicates
                    if !list.iter().any(|t| t.id == task.id) {
                        list.push(task);
                        tasks.set(list);
                    }
                }
                ProgressEvent::Update {
                    id,
                    percentage,
                    message,
                } => {
                    let mut list = tasks.read().clone();
                    if let Some(t) = list.iter_mut().find(|t| t.id == id) {
                        t.percentage = percentage.clamp(0.0, 100.0);
                        t.message = message;
                        t.updated_at = Utc::now();
                    }
                    tasks.set(list);
                }
                ProgressEvent::Complete { id, saved_path } => {
                    let mut list = tasks.read().clone();
                    if let Some(t) = list.iter_mut().find(|t| t.id == id) {
                        t.percentage = 100.0;
                        t.message = "Export completed successfully".to_string();
                        t.updated_at = Utc::now();
                        t.status = ExportTaskStatus::Completed { saved_path };
                    }
                    tasks.set(list);
                }
                ProgressEvent::Fail { id, error } => {
                    let mut list = tasks.read().clone();
                    if let Some(t) = list.iter_mut().find(|t| t.id == id) {
                        t.message = format!("Export failed: {error}");
                        t.updated_at = Utc::now();
                        t.status = ExportTaskStatus::Failed { error };
                    }
                    tasks.set(list);
                }
                ProgressEvent::ClearAll => {
                    tasks.set(Vec::new());
                }
            }
        }
    });

    rsx! { {children} }
}

/// Hook to access centralized export progress state
pub fn use_export_progress() -> ExportProgressContext {
    use_context::<ExportProgressContext>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_kind_labels() {
        let id = Uuid::new_v4();
        assert!(ExportKind::Course(id).label().contains(&id.to_string()));
        assert!(ExportKind::Plan(id).label().contains(&id.to_string()));
        assert!(ExportKind::Notes(id).label().contains(&id.to_string()));
        assert_eq!(ExportKind::AllData.label(), "All Data");
        assert_eq!(ExportKind::Other("X".into()).label(), "X");
    }

    #[test]
    fn reporter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        // UnboundedSender is Send+Sync, reporter should be too.
        assert_send_sync::<ExportProgressReporter>();
    }
}
