//! Core domain types persisted to disk and exchanged with the UI.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};

/// Explicit importance level. P1 is the most important.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    P1,
    P2,
    P3,
    P4,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::P3
    }
}

impl Priority {
    pub fn label(self) -> &'static str {
        match self {
            Priority::P1 => "Critical",
            Priority::P2 => "High",
            Priority::P3 => "Medium",
            Priority::P4 => "Low",
        }
    }

    /// Important tasks are the top half of the scale (Eisenhower "important" axis).
    pub fn is_important(self) -> bool {
        matches!(self, Priority::P1 | Priority::P2)
    }

    pub fn weight(self) -> f64 {
        match self {
            Priority::P1 => 40.0,
            Priority::P2 => 30.0,
            Priority::P3 => 18.0,
            Priority::P4 => 8.0,
        }
    }

    /// Accepts "p1", "1", "critical", "high", "medium", "med", "low", "urgent" ...
    pub fn parse_loose(s: &str) -> Option<Priority> {
        let s = s.trim().to_ascii_lowercase();
        let bare = s.trim_start_matches('!');
        match bare {
            "p1" | "1" | "critical" | "crit" | "urgent" | "top" => Some(Priority::P1),
            "p2" | "2" | "high" | "hi" | "important" => Some(Priority::P2),
            "p3" | "3" | "medium" | "med" | "normal" => Some(Priority::P3),
            "p4" | "4" | "low" | "someday" | "minor" => Some(Priority::P4),
            "" => match s.len() {
                3.. => Some(Priority::P1),
                2 => Some(Priority::P2),
                1 => Some(Priority::P3),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecurrenceKind {
    Daily,
    Weekdays,
    Weekly,
    Monthly,
    Yearly,
}

fn default_interval() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recurrence {
    pub kind: RecurrenceKind,
    /// Every N units (days, weeks, months, years). Ignored for `Weekdays`.
    #[serde(default = "default_interval")]
    pub interval: u32,
    /// For weekly recurrences: 0 = Monday ... 6 = Sunday.
    #[serde(default)]
    pub weekday: Option<u8>,
}

impl Recurrence {
    pub fn label(&self) -> String {
        const DAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
        let n = self.interval.max(1);
        match self.kind {
            RecurrenceKind::Daily => {
                if n == 1 { "Every day".into() } else { format!("Every {n} days") }
            }
            RecurrenceKind::Weekdays => "Every weekday".into(),
            RecurrenceKind::Weekly => {
                let day = self.weekday.and_then(|d| DAYS.get(d as usize)).copied();
                match (n, day) {
                    (1, Some(d)) => format!("Every {d}"),
                    (1, None) => "Every week".into(),
                    (n, Some(d)) => format!("Every {n} weeks on {d}"),
                    (n, None) => format!("Every {n} weeks"),
                }
            }
            RecurrenceKind::Monthly => {
                if n == 1 { "Every month".into() } else { format!("Every {n} months") }
            }
            RecurrenceKind::Yearly => {
                if n == 1 { "Every year".into() } else { format!("Every {n} years") }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subtask {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub priority: Priority,
    /// Hard deadline (date part).
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    /// Optional time-of-day for the deadline.
    #[serde(default)]
    pub due_time: Option<NaiveTime>,
    /// The day the user plans to work on it ("Today" plan).
    #[serde(default)]
    pub scheduled: Option<NaiveDate>,
    #[serde(default)]
    pub estimate_minutes: Option<u32>,
    #[serde(default)]
    pub time_spent_minutes: u32,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub sort_order: i64,
}

impl Task {
    pub fn is_open(&self) -> bool {
        self.completed_at.is_none() && self.deleted_at.is_none()
    }

    pub fn is_done(&self) -> bool {
        self.completed_at.is_some() && self.deleted_at.is_none()
    }

    pub fn subtasks_done(&self) -> usize {
        self.subtasks.iter().filter(|s| s.done).count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default = "default_color")]
    pub color: String,
    #[serde(default)]
    pub description: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub sort_order: i64,
}

fn default_color() -> String {
    "#6366f1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_user_name")]
    pub user_name: String,
    #[serde(default = "default_workspace")]
    pub workspace_name: String,
    /// How many minutes of focused work fit in a day (used by the planner).
    #[serde(default = "default_capacity")]
    pub daily_capacity_minutes: u32,
    /// A task due within this many days counts as "urgent" in the matrix.
    #[serde(default = "default_urgent_window")]
    pub urgent_window_days: i64,
    #[serde(default = "default_pomodoro")]
    pub pomodoro_minutes: u32,
    /// Estimate assumed for tasks without one when planning the day.
    #[serde(default = "default_estimate")]
    pub default_estimate_minutes: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Dark palette: "soft" (tinted surfaces) or "plain" (pure black).
    #[serde(default = "default_dark_style")]
    pub dark_style: String,
    /// Accent colour: one of [`ACCENTS`] or a custom `#rrggbb` value.
    #[serde(default = "default_accent")]
    pub accent: String,
    /// Show the on-device AI button in the add bar.
    #[serde(default)]
    pub ai_enabled: bool,
    /// Which downloaded model to use, see `ai::MODELS`.
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    /// Rewrite every quick-add sentence automatically (instead of on demand).
    #[serde(default)]
    pub ai_auto: bool,
}

fn default_ai_model() -> String {
    "qwen2.5-0.5b".into()
}

pub const DARK_STYLES: [&str; 2] = ["soft", "plain"];
pub const ACCENTS: [&str; 9] = ["indigo", "blue", "violet", "teal", "emerald", "amber", "orange", "rose", "neutral"];

fn default_user_name() -> String {
    "You".into()
}
fn default_workspace() -> String {
    "My Workspace".into()
}
fn default_capacity() -> u32 {
    360
}
fn default_urgent_window() -> i64 {
    2
}
fn default_pomodoro() -> u32 {
    25
}
fn default_estimate() -> u32 {
    30
}
fn default_theme() -> String {
    "system".into()
}
fn default_dark_style() -> String {
    "soft".into()
}
fn default_accent() -> String {
    "indigo".into()
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            user_name: default_user_name(),
            workspace_name: default_workspace(),
            daily_capacity_minutes: default_capacity(),
            urgent_window_days: default_urgent_window(),
            pomodoro_minutes: default_pomodoro(),
            default_estimate_minutes: default_estimate(),
            theme: default_theme(),
            dark_style: default_dark_style(),
            accent: default_accent(),
            ai_enabled: false,
            ai_model: default_ai_model(),
            ai_auto: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub id: String,
    pub task_id: Option<String>,
    pub minutes: u32,
    pub ended_at: DateTime<Utc>,
}

/// Everything the app persists. Serialized as one JSON document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub focus_sessions: Vec<FocusSession>,
}

fn default_version() -> u32 {
    1
}

impl Default for Database {
    fn default() -> Self {
        Database {
            version: 1,
            settings: Settings::default(),
            projects: Vec::new(),
            tasks: Vec::new(),
            focus_sessions: Vec::new(),
        }
    }
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
