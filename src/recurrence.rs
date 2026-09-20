//! Computes the next occurrence of a recurring task.

use crate::model::{Recurrence, RecurrenceKind};
use chrono::{Datelike, Days, Months, NaiveDate, Weekday};

fn weekday_index(d: Weekday) -> u8 {
    d.num_days_from_monday() as u8
}

/// Next occurrence strictly after `from`, and never before `today`
/// (completing a long-overdue daily task yields "today"/"tomorrow", not a pile of past dates).
pub fn next_occurrence(rec: &Recurrence, from: NaiveDate, today: NaiveDate) -> NaiveDate {
    let n = rec.interval.max(1) as u64;
    let mut date = from;
    for _ in 0..2000 {
        let next = step(rec, date, n);
        if next <= date {
            break;
        }
        date = next;
        if date >= today {
            return date;
        }
    }
    date
}

fn step(rec: &Recurrence, date: NaiveDate, n: u64) -> NaiveDate {
    match rec.kind {
        RecurrenceKind::Daily => date.checked_add_days(Days::new(n)).unwrap_or(date),
        RecurrenceKind::Weekdays => {
            let mut d = date.checked_add_days(Days::new(1)).unwrap_or(date);
            while matches!(d.weekday(), Weekday::Sat | Weekday::Sun) {
                d = d.checked_add_days(Days::new(1)).unwrap_or(d);
            }
            d
        }
        RecurrenceKind::Weekly => match rec.weekday {
            Some(target) if target < 7 => {
                // Advance to the next `target` weekday, honouring the interval in weeks.
                let cur = weekday_index(date.weekday());
                let mut delta = (target as i64 - cur as i64).rem_euclid(7) as u64;
                if delta == 0 {
                    delta = 7 * n;
                } else if n > 1 {
                    delta += 7 * (n - 1);
                }
                date.checked_add_days(Days::new(delta)).unwrap_or(date)
            }
            _ => date.checked_add_days(Days::new(7 * n)).unwrap_or(date),
        },
        RecurrenceKind::Monthly => date.checked_add_months(Months::new(n as u32)).unwrap_or(date),
        RecurrenceKind::Yearly => date.checked_add_months(Months::new(12 * n as u32)).unwrap_or(date),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn daily_from_overdue_lands_on_today() {
        let rec = Recurrence { kind: RecurrenceKind::Daily, interval: 1, weekday: None };
        assert_eq!(next_occurrence(&rec, d(2026, 9, 1), d(2026, 9, 10)), d(2026, 9, 10));
        assert_eq!(next_occurrence(&rec, d(2026, 9, 10), d(2026, 9, 10)), d(2026, 9, 11));
    }

    #[test]
    fn weekdays_skip_weekend() {
        let rec = Recurrence { kind: RecurrenceKind::Weekdays, interval: 1, weekday: None };
        // 2026-09-11 is a Friday.
        assert_eq!(next_occurrence(&rec, d(2026, 9, 11), d(2026, 9, 11)), d(2026, 9, 14));
    }

    #[test]
    fn weekly_on_weekday() {
        let rec = Recurrence { kind: RecurrenceKind::Weekly, interval: 1, weekday: Some(0) };
        // From Thursday 2026-09-10, next Monday is 09-14.
        assert_eq!(next_occurrence(&rec, d(2026, 9, 10), d(2026, 9, 10)), d(2026, 9, 14));
        let rec2 = Recurrence { kind: RecurrenceKind::Weekly, interval: 2, weekday: Some(0) };
        assert_eq!(next_occurrence(&rec2, d(2026, 9, 14), d(2026, 9, 14)), d(2026, 9, 28));
    }

    #[test]
    fn monthly_clamps_day() {
        let rec = Recurrence { kind: RecurrenceKind::Monthly, interval: 1, weekday: None };
        assert_eq!(next_occurrence(&rec, d(2026, 1, 31), d(2026, 1, 31)), d(2026, 2, 28));
    }
}
