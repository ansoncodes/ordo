//! Productivity analytics computed on demand from the in-memory database.

use crate::model::{Database, Priority};
use chrono::{DateTime, Datelike, Days, Local, NaiveDate, Utc};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Debug, Serialize)]
pub struct DailyPoint {
    pub date: NaiveDate,
    pub label: String,
    pub completed: u32,
    pub created: u32,
}

#[derive(Debug, Serialize)]
pub struct PriorityStat {
    pub priority: Priority,
    pub label: &'static str,
    pub open: u32,
    pub done: u32,
}

#[derive(Debug, Serialize)]
pub struct ProjectStat {
    pub id: Option<String>,
    pub name: String,
    pub color: String,
    pub open: u32,
    pub done: u32,
    pub overdue: u32,
}

#[derive(Debug, Serialize)]
pub struct WeekdayStat {
    pub label: &'static str,
    pub completed: u32,
}

#[derive(Debug, Serialize)]
pub struct EstimateStat {
    pub estimated_minutes: u32,
    pub spent_minutes: u32,
    pub tasks: u32,
}

#[derive(Debug, Serialize)]
pub struct Analytics {
    pub completed_today: u32,
    pub completed_week: u32,
    pub completed_total: u32,
    pub open_total: u32,
    pub overdue: u32,
    pub due_today: u32,
    pub planned_today: u32,
    pub streak_days: u32,
    pub best_streak: u32,
    /// Percentage of tasks (completed in the last 30 days, with a deadline) finished on time.
    pub on_time_rate: Option<u32>,
    pub avg_completion_days: Option<f64>,
    pub focus_minutes_week: u32,
    pub focus_minutes_total: u32,
    pub focus_sessions_week: u32,
    pub daily: Vec<DailyPoint>,
    pub by_priority: Vec<PriorityStat>,
    pub by_project: Vec<ProjectStat>,
    pub by_weekday: Vec<WeekdayStat>,
    pub estimate: EstimateStat,
}

fn local_date(dt: DateTime<Utc>) -> NaiveDate {
    dt.with_timezone(&Local).date_naive()
}

pub fn compute(db: &Database, today: NaiveDate) -> Analytics {
    let week_start = today - Days::new(6);
    let month_start = today - Days::new(29);
    let eight_weeks = today - Days::new(55);

    let live: Vec<_> = db.tasks.iter().filter(|t| t.deleted_at.is_none()).collect();
    let open: Vec<_> = live.iter().copied().filter(|t| t.completed_at.is_none()).collect();
    let done: Vec<_> = live.iter().copied().filter(|t| t.completed_at.is_some()).collect();

    let completion_dates: Vec<NaiveDate> = done.iter().filter_map(|t| t.completed_at.map(local_date)).collect();

    let completed_today = completion_dates.iter().filter(|d| **d == today).count() as u32;
    let completed_week = completion_dates.iter().filter(|d| **d >= week_start && **d <= today).count() as u32;

    let overdue = open.iter().filter(|t| t.due_date.map(|d| d < today).unwrap_or(false)).count() as u32;
    let due_today = open.iter().filter(|t| t.due_date == Some(today)).count() as u32;
    let planned_today = open.iter().filter(|t| t.scheduled.map(|s| s <= today).unwrap_or(false)).count() as u32;

    // Streaks.
    let days: BTreeSet<NaiveDate> = completion_dates.iter().copied().collect();
    let mut streak_days = 0;
    let mut cursor = if days.contains(&today) { today } else { today - Days::new(1) };
    while days.contains(&cursor) {
        streak_days += 1;
        cursor = cursor - Days::new(1);
    }
    let mut best_streak = 0u32;
    let mut run = 0u32;
    let mut prev: Option<NaiveDate> = None;
    for d in &days {
        run = match prev {
            Some(p) if *d == p + Days::new(1) => run + 1,
            _ => 1,
        };
        best_streak = best_streak.max(run);
        prev = Some(*d);
    }

    // On-time rate and cycle time over the last 30 days.
    let recent_done: Vec<_> = done
        .iter()
        .copied()
        .filter(|t| t.completed_at.map(|c| local_date(c) >= month_start).unwrap_or(false))
        .collect();
    let with_due: Vec<_> = recent_done.iter().copied().filter(|t| t.due_date.is_some()).collect();
    let on_time_rate = if with_due.is_empty() {
        None
    } else {
        let on_time = with_due
            .iter()
            .filter(|t| local_date(t.completed_at.unwrap()) <= t.due_date.unwrap())
            .count();
        Some(((on_time as f64 / with_due.len() as f64) * 100.0).round() as u32)
    };
    let avg_completion_days = if recent_done.is_empty() {
        None
    } else {
        let total: f64 = recent_done
            .iter()
            .map(|t| (t.completed_at.unwrap() - t.created_at).num_minutes() as f64 / 1440.0)
            .sum();
        Some((total / recent_done.len() as f64 * 10.0).round() / 10.0)
    };

    // Daily series (last 14 days).
    let mut daily = Vec::new();
    for offset in (0..14).rev() {
        let date = today - Days::new(offset);
        let completed = completion_dates.iter().filter(|d| **d == date).count() as u32;
        let created = live.iter().filter(|t| local_date(t.created_at) == date).count() as u32;
        daily.push(DailyPoint { date, label: date.format("%a %-d").to_string(), completed, created });
    }

    // By priority.
    let by_priority = [Priority::P1, Priority::P2, Priority::P3, Priority::P4]
        .into_iter()
        .map(|p| PriorityStat {
            priority: p,
            label: p.label(),
            open: open.iter().filter(|t| t.priority == p).count() as u32,
            done: done.iter().filter(|t| t.priority == p).count() as u32,
        })
        .collect();

    // By project (including Inbox).
    let mut by_project: Vec<ProjectStat> = Vec::new();
    let mut project_ids: Vec<Option<String>> = vec![None];
    project_ids.extend(db.projects.iter().map(|p| Some(p.id.clone())));
    for pid in project_ids {
        let (name, color) = match &pid {
            None => ("Inbox".to_string(), "#94a3b8".to_string()),
            Some(id) => {
                let p = db.projects.iter().find(|p| &p.id == id).unwrap();
                (p.name.clone(), p.color.clone())
            }
        };
        let open_n = open.iter().filter(|t| t.project_id == pid).count() as u32;
        let done_n = done.iter().filter(|t| t.project_id == pid).count() as u32;
        let overdue_n = open
            .iter()
            .filter(|t| t.project_id == pid && t.due_date.map(|d| d < today).unwrap_or(false))
            .count() as u32;
        if open_n + done_n > 0 {
            by_project.push(ProjectStat { id: pid, name, color, open: open_n, done: done_n, overdue: overdue_n });
        }
    }

    // Weekday productivity over the last 8 weeks.
    let mut weekday_counts = [0u32; 7];
    for d in completion_dates.iter().filter(|d| **d >= eight_weeks) {
        weekday_counts[d.weekday().num_days_from_monday() as usize] += 1;
    }
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let by_weekday = labels
        .iter()
        .zip(weekday_counts.iter())
        .map(|(label, completed)| WeekdayStat { label, completed: *completed })
        .collect();

    // Estimate accuracy: completed tasks that have both an estimate and logged time.
    let mut estimate = EstimateStat { estimated_minutes: 0, spent_minutes: 0, tasks: 0 };
    for t in done.iter().filter(|t| t.estimate_minutes.is_some() && t.time_spent_minutes > 0) {
        estimate.estimated_minutes += t.estimate_minutes.unwrap_or(0);
        estimate.spent_minutes += t.time_spent_minutes;
        estimate.tasks += 1;
    }

    let focus_week: Vec<_> = db.focus_sessions.iter().filter(|s| local_date(s.ended_at) >= week_start).collect();
    let focus_minutes_week = focus_week.iter().map(|s| s.minutes).sum();
    let focus_sessions_week = focus_week.len() as u32;
    let focus_minutes_total = db.focus_sessions.iter().map(|s| s.minutes).sum();

    Analytics {
        completed_today,
        completed_week,
        completed_total: done.len() as u32,
        open_total: open.len() as u32,
        overdue,
        due_today,
        planned_today,
        streak_days,
        best_streak,
        on_time_rate,
        avg_completion_days,
        focus_minutes_week,
        focus_minutes_total,
        focus_sessions_week,
        daily,
        by_priority,
        by_project,
        by_weekday,
        estimate,
    }
}
