//! Demo workspace created on first launch so the product is explorable immediately.

use crate::model::*;
use chrono::{Days, Duration, NaiveDate, NaiveTime, Utc};

pub const PROJECT_COLORS: [&str; 8] =
    ["#6366f1", "#10b981", "#f59e0b", "#ef4444", "#0ea5e9", "#8b5cf6", "#ec4899", "#14b8a6"];

struct Spec<'a> {
    title: &'a str,
    notes: &'a str,
    project: Option<usize>,
    priority: Priority,
    due_in: Option<i64>,
    due_time: Option<(u32, u32)>,
    scheduled_in: Option<i64>,
    estimate: Option<u32>,
    tags: &'a [&'a str],
    subtasks: &'a [(&'a str, bool)],
    recurrence: Option<Recurrence>,
    starred: bool,
    age_days: i64,
    done_days_ago: Option<i64>,
    spent: u32,
}

impl Default for Spec<'_> {
    fn default() -> Self {
        Spec {
            title: "",
            notes: "",
            project: None,
            priority: Priority::P3,
            due_in: None,
            due_time: None,
            scheduled_in: None,
            estimate: None,
            tags: &[],
            subtasks: &[],
            recurrence: None,
            starred: false,
            age_days: 1,
            done_days_ago: None,
            spent: 0,
        }
    }
}

fn weekly(weekday: u8) -> Option<Recurrence> {
    Some(Recurrence { kind: RecurrenceKind::Weekly, interval: 1, weekday: Some(weekday) })
}

pub fn demo_database(today: NaiveDate) -> Database {
    let now = Utc::now();
    let projects: Vec<Project> = [
        ("Product Launch", "#6366f1", "Everything needed to ship v2 to customers."),
        ("Personal", "#10b981", "Life admin, health and home."),
        ("Learning", "#f59e0b", "Skills I am deliberately building."),
    ]
    .iter()
    .enumerate()
    .map(|(i, (name, color, desc))| Project {
        id: new_id(),
        name: name.to_string(),
        color: color.to_string(),
        description: desc.to_string(),
        created_at: now - Duration::days(30),
        sort_order: i as i64,
    })
    .collect();

    let specs = vec![
        Spec {
            title: "Welcome to Ordo - click me to see how tasks work",
            notes: "Ordo ranks your work with a focus score (priority + deadline + planning + age + effort).\n\nTry:\n- The quick-add bar: \"Call Sam tomorrow 3pm !p1 #Personal ~30m\"\n- The Priority Matrix view (drag tasks between quadrants)\n- \"Plan my day\" in Today to fill your capacity with the best next tasks\n- Ctrl+K for the command palette",
            priority: Priority::P2,
            scheduled_in: Some(0),
            estimate: Some(10),
            tags: &["onboarding"],
            subtasks: &[("Add a task with the quick-add bar", false), ("Open the Priority Matrix", false), ("Start a focus session", false)],
            starred: true,
            age_days: 0,
            ..Default::default()
        },
        Spec {
            title: "Finalize Q4 launch pricing",
            notes: "Compare against the three competitor tiers. Decision needed before the announcement post goes out.",
            project: Some(0),
            priority: Priority::P1,
            due_in: Some(0),
            due_time: Some((16, 0)),
            estimate: Some(60),
            tags: &["planning"],
            age_days: 6,
            ..Default::default()
        },
        Spec {
            title: "Review open pull requests",
            project: Some(0),
            priority: Priority::P2,
            due_in: Some(0),
            estimate: Some(30),
            tags: &["engineering"],
            recurrence: Some(Recurrence { kind: RecurrenceKind::Weekdays, interval: 1, weekday: None }),
            age_days: 20,
            ..Default::default()
        },
        Spec {
            title: "Write launch announcement blog post",
            notes: "Audience: existing customers first, then the public changelog.",
            project: Some(0),
            priority: Priority::P2,
            due_in: Some(3),
            estimate: Some(120),
            tags: &["writing", "marketing"],
            subtasks: &[("Outline the key messages", true), ("Write the first draft", false), ("Get feedback from design", false)],
            age_days: 4,
            ..Default::default()
        },
        Spec {
            title: "Prepare demo for stakeholder meeting",
            project: Some(0),
            priority: Priority::P1,
            due_in: Some(2),
            due_time: Some((10, 0)),
            estimate: Some(90),
            tags: &["meeting"],
            subtasks: &[("Reset the demo workspace", false), ("Rehearse the 10-minute walkthrough", false)],
            age_days: 3,
            ..Default::default()
        },
        Spec {
            title: "Book dentist appointment",
            project: Some(1),
            priority: Priority::P3,
            due_in: Some(-2),
            estimate: Some(10),
            tags: &["health"],
            age_days: 12,
            ..Default::default()
        },
        Spec {
            title: "Pay electricity bill",
            project: Some(1),
            priority: Priority::P1,
            due_in: Some(1),
            estimate: Some(5),
            tags: &["finance"],
            recurrence: Some(Recurrence { kind: RecurrenceKind::Monthly, interval: 1, weekday: None }),
            age_days: 25,
            ..Default::default()
        },
        Spec {
            title: "Grocery run for the week",
            project: Some(1),
            priority: Priority::P3,
            due_in: Some(0),
            scheduled_in: Some(0),
            estimate: Some(45),
            tags: &["errands"],
            age_days: 1,
            ..Default::default()
        },
        Spec {
            title: "Plan the weekend hike",
            project: Some(1),
            priority: Priority::P4,
            due_in: Some(9),
            estimate: Some(20),
            tags: &["outdoors"],
            age_days: 2,
            ..Default::default()
        },
        Spec {
            title: "Renew passport",
            notes: "Expires in 4 months. Needs new photos.",
            project: Some(1),
            priority: Priority::P3,
            due_in: Some(21),
            estimate: Some(40),
            age_days: 8,
            ..Default::default()
        },
        Spec {
            title: "Practice Rust: implement an LRU cache",
            project: Some(2),
            priority: Priority::P3,
            scheduled_in: Some(0),
            estimate: Some(90),
            tags: &["rust"],
            age_days: 5,
            ..Default::default()
        },
        Spec {
            title: "Read Deep Work, chapter 3",
            project: Some(2),
            priority: Priority::P4,
            estimate: Some(45),
            tags: &["reading"],
            age_days: 15,
            ..Default::default()
        },
        Spec {
            title: "Weekly review",
            notes: "Clear the inbox, check overdue tasks, plan next week.",
            priority: Priority::P2,
            due_in: Some(1),
            estimate: Some(30),
            tags: &["ritual"],
            recurrence: weekly(4),
            age_days: 40,
            ..Default::default()
        },
        Spec {
            title: "Reply to Maya about the conference talk",
            priority: Priority::P2,
            due_in: Some(1),
            estimate: Some(15),
            tags: &["email"],
            age_days: 2,
            ..Default::default()
        },
        Spec {
            title: "Sketch ideas for the mobile onboarding flow",
            project: Some(0),
            priority: Priority::P4,
            estimate: Some(60),
            tags: &["design"],
            age_days: 9,
            ..Default::default()
        },
        // Completed history for the analytics view.
        Spec { title: "Fix login redirect bug", project: Some(0), priority: Priority::P1, estimate: Some(45), tags: &["engineering"], age_days: 3, done_days_ago: Some(0), spent: 50, due_in: Some(0), ..Default::default() },
        Spec { title: "Update dependencies", project: Some(0), priority: Priority::P3, estimate: Some(30), age_days: 4, done_days_ago: Some(1), spent: 25, ..Default::default() },
        Spec { title: "Set up CI pipeline", project: Some(0), priority: Priority::P2, estimate: Some(120), tags: &["engineering"], age_days: 8, done_days_ago: Some(1), spent: 150, due_in: Some(-1), ..Default::default() },
        Spec { title: "Send onboarding email to new customers", project: Some(0), priority: Priority::P2, estimate: Some(20), tags: &["marketing"], age_days: 5, done_days_ago: Some(2), spent: 20, due_in: Some(-2), ..Default::default() },
        Spec { title: "Draft the roadmap for next quarter", project: Some(0), priority: Priority::P1, estimate: Some(90), tags: &["planning"], age_days: 10, done_days_ago: Some(3), spent: 110, due_in: Some(-3), ..Default::default() },
        Spec { title: "Order a standing desk", project: Some(1), priority: Priority::P4, estimate: Some(15), age_days: 12, done_days_ago: Some(5), ..Default::default() },
        Spec { title: "Call the insurance company", project: Some(1), priority: Priority::P3, estimate: Some(20), age_days: 9, done_days_ago: Some(6), spent: 15, due_in: Some(-8), ..Default::default() },
        Spec { title: "Finish the ownership and borrowing chapter", project: Some(2), priority: Priority::P3, estimate: Some(60), tags: &["rust"], age_days: 11, done_days_ago: Some(6), spent: 70, ..Default::default() },
        Spec { title: "Customer interview with Northwind", project: Some(0), priority: Priority::P2, estimate: Some(45), age_days: 9, done_days_ago: Some(7), spent: 45, due_in: Some(-7), ..Default::default() },
        Spec { title: "Clean out the garage", project: Some(1), priority: Priority::P4, estimate: Some(120), age_days: 14, done_days_ago: Some(9), ..Default::default() },
    ];

    let mut tasks = Vec::new();
    for (i, s) in specs.into_iter().enumerate() {
        let created = now - Duration::days(s.age_days) - Duration::minutes(i as i64 * 7);
        let completed_at = s.done_days_ago.map(|d| now - Duration::days(d) - Duration::hours(2));
        tasks.push(Task {
            id: new_id(),
            title: s.title.to_string(),
            notes: s.notes.to_string(),
            project_id: s.project.map(|p| projects[p].id.clone()),
            tags: s.tags.iter().map(|t| t.to_string()).collect(),
            priority: s.priority,
            due_date: s.due_in.map(|d| shift(today, d)),
            due_time: s.due_time.and_then(|(h, m)| NaiveTime::from_hms_opt(h, m, 0)),
            scheduled: s.scheduled_in.map(|d| shift(today, d)),
            estimate_minutes: s.estimate,
            time_spent_minutes: s.spent,
            subtasks: s
                .subtasks
                .iter()
                .map(|(title, done)| Subtask { id: new_id(), title: title.to_string(), done: *done })
                .collect(),
            recurrence: s.recurrence,
            starred: s.starred,
            completed_at,
            deleted_at: None,
            created_at: created,
            updated_at: completed_at.unwrap_or(created),
            sort_order: i as i64,
        });
    }

    let focus_sessions = [(0, 25), (0, 25), (1, 50), (2, 25), (3, 25), (3, 25), (6, 25)]
        .iter()
        .map(|(days_ago, minutes)| FocusSession {
            id: new_id(),
            task_id: None,
            minutes: *minutes,
            ended_at: now - Duration::days(*days_ago) - Duration::hours(3),
        })
        .collect();

    Database { version: 1, settings: Settings::default(), projects, tasks, focus_sessions }
}

fn shift(date: NaiveDate, days: i64) -> NaiveDate {
    if days >= 0 {
        date + Days::new(days as u64)
    } else {
        date - Days::new((-days) as u64)
    }
}
