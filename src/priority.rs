//! The priority engine: turns a task's fields into an explainable focus score
//! and an Eisenhower-matrix quadrant.

use crate::model::{Settings, Task};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Quadrant {
    /// Important and urgent: do first.
    Q1,
    /// Important, not urgent: schedule.
    Q2,
    /// Urgent, not important: do quickly / delegate.
    Q3,
    /// Neither: someday.
    Q4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DueStatus {
    Overdue,
    Today,
    Tomorrow,
    Soon,
    Later,
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct Scoring {
    /// 0-100 focus score, higher = do sooner.
    pub score: u32,
    pub reasons: Vec<String>,
    pub urgent: bool,
    pub important: bool,
    pub quadrant: Quadrant,
    pub due_status: DueStatus,
    pub days_until_due: Option<i64>,
}

const RAW_MAX: f64 = 130.0;

pub fn evaluate(task: &Task, today: NaiveDate, now: DateTime<Utc>, settings: &Settings) -> Scoring {
    let mut raw = 0.0;
    let mut reasons = Vec::new();

    // 1. Explicit priority.
    raw += task.priority.weight();
    reasons.push(format!("{} priority", task.priority.label()));

    // 2. Deadline proximity.
    let days_until_due = task.due_date.map(|d| (d - today).num_days());
    let window = settings.urgent_window_days.max(0);
    let due_status = match days_until_due {
        Some(d) if d < 0 => DueStatus::Overdue,
        Some(0) => DueStatus::Today,
        Some(1) => DueStatus::Tomorrow,
        Some(d) if d <= 7 => DueStatus::Soon,
        Some(_) => DueStatus::Later,
        None => DueStatus::None,
    };
    match days_until_due {
        Some(d) if d < 0 => {
            let overdue = -d;
            raw += 35.0 + (overdue as f64 * 3.0).min(15.0);
            reasons.push(if overdue == 1 { "Overdue by 1 day".into() } else { format!("Overdue by {overdue} days") });
        }
        Some(0) => {
            raw += 30.0;
            reasons.push("Due today".into());
        }
        Some(1) => {
            raw += 22.0;
            reasons.push("Due tomorrow".into());
        }
        Some(d) if d <= window => {
            raw += 18.0;
            reasons.push(format!("Due in {d} days"));
        }
        Some(d) if d <= 7 => {
            raw += 10.0;
            reasons.push(format!("Due in {d} days"));
        }
        Some(d) if d <= 14 => {
            raw += 5.0;
            reasons.push("Due within two weeks".into());
        }
        Some(_) => raw += 1.0,
        None => {}
    }

    // 3. Planned for today (or earlier and still open).
    let planned_today = task.scheduled.map(|s| s <= today).unwrap_or(false);
    if let (true, Some(scheduled)) = (planned_today, task.scheduled) {
        raw += 20.0;
        let behind = (today - scheduled).num_days();
        if behind > 0 {
            reasons.push(format!("Planned {behind} days ago"));
        } else {
            reasons.push("Planned for today".into());
        }
    }

    // 4. Starred.
    if task.starred {
        raw += 12.0;
        reasons.push("Starred".into());
    }

    // 5. Age: old tasks slowly float up so nothing rots forever.
    let age_days = (now - task.created_at).num_days().max(0);
    raw += (age_days.min(14) as f64) * 0.5;
    if age_days >= 7 {
        reasons.push(format!("Waiting {age_days} days"));
    }

    // 6. Quick wins.
    match task.estimate_minutes {
        Some(m) if m <= 15 => {
            raw += 4.0;
            reasons.push("Quick win (15 min or less)".into());
        }
        Some(m) if m <= 30 => {
            raw += 2.0;
            reasons.push("Short task (30 min or less)".into());
        }
        _ => {}
    }

    // 7. Momentum: partially completed checklists.
    if !task.subtasks.is_empty() {
        let done = task.subtasks_done();
        if done > 0 && done < task.subtasks.len() {
            raw += 3.0;
            reasons.push(format!("In progress ({done}/{})", task.subtasks.len()));
        }
    }

    let score = ((raw / RAW_MAX) * 100.0).round().clamp(0.0, 100.0) as u32;

    let important = task.priority.is_important();
    let urgent = matches!(due_status, DueStatus::Overdue | DueStatus::Today)
        || days_until_due.map(|d| (0..=window).contains(&d)).unwrap_or(false)
        || planned_today;
    let quadrant = match (important, urgent) {
        (true, true) => Quadrant::Q1,
        (true, false) => Quadrant::Q2,
        (false, true) => Quadrant::Q3,
        (false, false) => Quadrant::Q4,
    };

    Scoring { score, reasons, urgent, important, quadrant, due_status, days_until_due }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{new_id, Priority};

    fn task(priority: Priority, due: Option<NaiveDate>) -> Task {
        let now = Utc::now();
        Task {
            id: new_id(),
            title: "t".into(),
            notes: String::new(),
            project_id: None,
            tags: vec![],
            priority,
            due_date: due,
            due_time: None,
            scheduled: None,
            estimate_minutes: None,
            time_spent_minutes: 0,
            subtasks: vec![],
            recurrence: None,
            starred: false,
            completed_at: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
            sort_order: 0,
        }
    }

    #[test]
    fn overdue_critical_outranks_low_no_date() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 10).unwrap();
        let s = Settings::default();
        let a = evaluate(&task(Priority::P1, today.pred_opt()), today, Utc::now(), &s);
        let b = evaluate(&task(Priority::P4, None), today, Utc::now(), &s);
        assert!(a.score > b.score);
        assert_eq!(a.quadrant, Quadrant::Q1);
        assert_eq!(b.quadrant, Quadrant::Q4);
        assert_eq!(a.due_status, DueStatus::Overdue);
    }

    #[test]
    fn planned_today_counts_as_urgent() {
        let today = NaiveDate::from_ymd_opt(2026, 9, 10).unwrap();
        let mut t = task(Priority::P3, None);
        t.scheduled = Some(today);
        let s = evaluate(&t, today, Utc::now(), &Settings::default());
        assert!(s.urgent);
        assert_eq!(s.quadrant, Quadrant::Q3);
    }
}
