//! Natural-language quick-add parser.
//!
//! Turns `"Send invoice to Acme tomorrow 5pm !p1 #Work @finance ~45m every friday"`
//! into a title plus structured fields, and reports which pieces of the input
//! were recognised so the UI can highlight them live.

use crate::model::{Priority, Recurrence, RecurrenceKind};
use chrono::{Datelike, Days, Months, NaiveDate, NaiveTime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Token {
    pub kind: &'static str,
    pub text: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Parsed {
    pub title: String,
    /// Project name exactly as typed (or the matched known project name).
    pub project: Option<String>,
    pub tags: Vec<String>,
    pub priority: Option<Priority>,
    pub due_date: Option<NaiveDate>,
    pub due_time: Option<NaiveTime>,
    pub estimate_minutes: Option<u32>,
    pub recurrence: Option<Recurrence>,
    pub starred: bool,
    /// Text after `//`.
    pub notes: String,
    /// `^tomorrow`: the day the user plans to work on it.
    pub scheduled: Option<NaiveDate>,
    /// `+step` checklist items.
    pub subtasks: Vec<String>,
    pub tokens: Vec<Token>,
}

const PROJECT_CONNECTORS: [&str; 4] = ["in", "for", "under", "into"];
const ESTIMATE_CONNECTORS: [&str; 6] = ["in", "for", "takes", "about", "around", "approx"];

/// Keywords that tolerate typos ("tommorow", "fridy", "criticl"). Deliberately excludes
/// words with common real-word neighbours ("month" vs "mouth", "hours" vs "yours").
const FUZZY_VOCAB: [&str; 37] = [
    "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday", "january", "february", "august",
    "september", "october", "november", "december", "today", "tomorrow", "tonight", "yesterday", "weekend", "weekly",
    "monthly", "yearly", "daily", "annually", "fortnightly", "fortnight", "weekdays", "weekday", "every", "critical",
    "urgent",
    "important", "medium", "someday", "morning", "afternoon", "evening",
];
const TYPO_ALIASES: [(&str, &str); 16] = [
    ("wensday", "wednesday"),
    ("wendsday", "wednesday"),
    ("wednsday", "wednesday"),
    ("tommorrow", "tomorrow"),
    ("tommorow", "tomorrow"),
    ("tomorow", "tomorrow"),
    ("2moro", "tomorrow"),
    ("2morrow", "tomorrow"),
    ("tonite", "tonight"),
    ("thrusday", "thursday"),
    ("thurday", "thursday"),
    ("evry", "every"),
    ("nxt", "next"),
    ("mrng", "morning"),
    ("evng", "evening"),
    ("aftrnoon", "afternoon"),
];

const WEEKDAYS: [&str; 7] = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];
const WEEKDAY_ABBR: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
const MONTHS: [&str; 12] = [
    "january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november",
    "december",
];

struct Cursor<'a> {
    words: Vec<&'a str>,
    clean: Vec<String>,
    consumed: Vec<bool>,
    /// Word carried a `^` prefix (planned-for date marker).
    planned: Vec<bool>,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str, known_projects: &[String], known_tags: &[String]) -> Self {
        let words: Vec<&str> = input.split_whitespace().collect();
        let planned: Vec<bool> = words.iter().map(|w| w.len() > 1 && w.starts_with('^')).collect();
        // Typo tolerance: keywords, project-name words and tag names within a small edit distance.
        let mut vocab: Vec<String> = FUZZY_VOCAB.iter().map(|w| w.to_string()).collect();
        for name in known_projects {
            vocab.extend(name.to_lowercase().split_whitespace().filter(|w| w.chars().count() >= 5).map(String::from));
        }
        vocab.extend(known_tags.iter().map(|t| t.to_lowercase()));
        let clean = words
            .iter()
            .map(|w| {
                let base = if w.len() > 1 { w.trim_start_matches('^') } else { w };
                let lower = base.to_lowercase().trim_end_matches([',', '.', ';']).to_string();
                let prefix_len = lower.chars().take_while(|c| matches!(c, '#' | '@' | '!' | '~')).count();
                let (prefix, body) = lower.split_at(prefix_len);
                match fuzzy_correct(body, &vocab) {
                    Some(fixed) => format!("{prefix}{fixed}"),
                    None => lower.clone(),
                }
            })
            .collect();
        let consumed = vec![false; words.len()];
        Cursor { words, clean, consumed, planned }
    }

    fn at(&self, i: usize) -> Option<&str> {
        self.clean.get(i).map(|s| s.as_str())
    }

    fn original(&self, i: usize, n: usize) -> String {
        self.words[i..(i + n).min(self.words.len())].join(" ")
    }
}

pub fn parse(input: &str, today: NaiveDate, known_projects: &[String], known_tags: &[String]) -> Parsed {
    let mut out = Parsed::default();
    let (main, notes) = split_notes(input);
    let mut c = Cursor::new(main, known_projects, known_tags);
    if !notes.is_empty() {
        let preview: String = notes.chars().take(40).collect();
        out.tokens.push(Token { kind: "notes", text: format!("// {notes}"), value: preview });
        out.notes = notes;
    }
    let mut implied_due: Option<NaiveDate> = None;
    let mut implied_time: Option<NaiveTime> = None;
    let n = c.words.len();
    let mut i = 0;

    while i < n {
        let w = c.clean[i].clone();
        let raw = c.words[i];

        // #project (greedy match against known multi-word project names)
        if let Some(rest) = raw.strip_prefix('#') {
            if !rest.is_empty() && out.project.is_none() {
                let (name, len) = match_project(&c, i, known_projects);
                push_token(&mut out, &mut c, i, len, "project", name.clone());
                out.project = Some(name);
                i += len;
                continue;
            }
        }

        // @tag
        if raw.starts_with('@') {
            // `clean` is lower-cased and typo-corrected against known tags.
            let tag = c.clean[i].trim_start_matches('@').to_string();
            if !tag.is_empty() {
                push_token(&mut out, &mut c, i, 1, "tag", tag.clone());
                if !out.tags.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
                    out.tags.push(tag);
                }
                i += 1;
                continue;
            }
        }

        // Priority: !p1, !high, !!!, p2 ...
        if w.starts_with('!') || matches!(w.as_str(), "p1" | "p2" | "p3" | "p4") {
            if let Some(p) = Priority::parse_loose(&w) {
                push_token(&mut out, &mut c, i, 1, "priority", p.label().to_string());
                out.priority = Some(p);
                i += 1;
                continue;
            }
        }

        // Star: a lone "*" or "star!"
        if w == "*" || w == "★" {
            push_token(&mut out, &mut c, i, 1, "star", "Starred".into());
            out.starred = true;
            i += 1;
            continue;
        }

        // Planned-for date: ^tomorrow, ^next monday, "^ friday"
        if c.planned[i] || w == "^" {
            let j = if w == "^" { i + 1 } else { i };
            if let Some((date, len, _)) = parse_date(&c, j, today) {
                let total = (j - i) + len;
                push_token(&mut out, &mut c, i, total, "planned", format!("Plan {}", date.format("%a %-d %b")));
                out.scheduled = Some(date);
                i += total;
                continue;
            }
        }

        // Checklist steps: "+outline +draft the post"
        if let Some(rest) = raw.strip_prefix('+') {
            let mut step_words: Vec<&str> = Vec::new();
            if !rest.is_empty() {
                step_words.push(rest);
            }
            let mut j = i + 1;
            while j < n && !is_directive(c.words[j]) {
                step_words.push(c.words[j]);
                j += 1;
            }
            let step = step_words.join(" ").trim_end_matches([',', ';']).trim().to_string();
            if !step.is_empty() {
                push_token(&mut out, &mut c, i, j - i, "step", step.clone());
                out.subtasks.push(step);
                i = j;
                continue;
            }
        }

        // Project named in plain words: "hacking challenge project", "for Hacking Challenge"
        if out.project.is_none() {
            if let Some((name, start, len)) = match_project_phrase(&c, i, known_projects) {
                push_token(&mut out, &mut c, start, len, "project", name.clone());
                out.project = Some(name);
                i = start + len;
                continue;
            }
        }

        // Recurrence (must run before dates so "every monday" is not read as "monday").
        if let Some((rec, len, implied, time)) = parse_recurrence(&c, i, today) {
            push_token(&mut out, &mut c, i, len, "recurrence", rec.label());
            out.recurrence = Some(rec);
            implied_due = implied_due.or(implied);
            implied_time = implied_time.or(time);
            i += len;
            continue;
        }

        // Estimate: ~45m, 1h30m, 2 hours, "in 10 mins", "for about 1h" ...
        if let Some((minutes, len)) = parse_duration(&c, i) {
            let mut start = i;
            while start > 0 && !c.consumed[start - 1] && ESTIMATE_CONNECTORS.contains(&c.clean[start - 1].as_str()) {
                start -= 1;
            }
            push_token(&mut out, &mut c, start, len + (i - start), "estimate", format_minutes(minutes));
            out.estimate_minutes = Some(minutes);
            i += len;
            continue;
        }

        // Dates (with optional on/by/due prefix).
        if let Some((date, len, time)) = parse_date(&c, i, today) {
            push_token(&mut out, &mut c, i, len, "date", date.format("%a %-d %b").to_string());
            out.due_date = Some(date);
            implied_time = implied_time.or(time);
            i += len;
            continue;
        }

        // Times (with optional "at" prefix).
        if let Some((time, len)) = parse_time(&c, i) {
            push_token(&mut out, &mut c, i, len, "time", time.format("%H:%M").to_string());
            out.due_time = Some(time);
            i += len;
            continue;
        }

        i += 1;
    }

    // Post-processing.
    if let Some(rec) = out.recurrence.as_mut() {
        if rec.kind == RecurrenceKind::Weekly && rec.weekday.is_none() {
            if let Some(d) = out.due_date {
                rec.weekday = Some(d.weekday().num_days_from_monday() as u8);
            }
        }
        if out.due_date.is_none() {
            out.due_date = Some(implied_due.unwrap_or(today));
        }
    }
    if out.due_time.is_none() {
        out.due_time = implied_time;
    }
    if out.due_time.is_some() && out.due_date.is_none() {
        out.due_date = Some(today);
    }

    let mut title_words = Vec::new();
    for (idx, word) in c.words.iter().enumerate() {
        if !c.consumed[idx] {
            title_words.push(*word);
        }
    }
    let title = title_words.join(" ");
    // Drop dangling connector words left behind ("Pay rent on" -> "Pay rent").
    let title = title
        .trim()
        .trim_end_matches(|ch: char| ch == ',' || ch == '-' || ch == ':')
        .trim();
    let mut title = title.to_string();
    for suffix in [" on", " by", " at", " due", " in"] {
        if let Some(stripped) = title.strip_suffix(suffix) {
            title = stripped.to_string();
        }
    }
    out.title = title.trim().to_string();
    out
}

fn push_token(out: &mut Parsed, c: &mut Cursor, i: usize, len: usize, kind: &'static str, value: String) {
    for k in i..(i + len).min(c.words.len()) {
        c.consumed[k] = true;
    }
    out.tokens.push(Token { kind, text: c.original(i, len), value });
}

fn match_project(c: &Cursor, i: usize, known: &[String]) -> (String, usize) {
    let max_len = 4.min(c.words.len() - i);
    for len in (1..=max_len).rev() {
        // `clean` words are lower-cased and typo-corrected, so "#hacking chalenge" still matches.
        let candidate = c.clean[i..i + len].join(" ");
        let candidate = candidate.trim_start_matches('#');
        if let Some(name) = known.iter().find(|p| p.to_lowercase() == candidate) {
            return (name.clone(), len);
        }
    }
    let single = c.words[i].trim_start_matches('#').trim_end_matches([',', '.', ';']);
    (single.replace('_', " "), 1)
}

/// Optimal-string-alignment edit distance (insert, delete, substitute, adjacent transposition).
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (n, m) = (a.len(), b.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for j in 0..=m {
        d[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
            }
        }
    }
    d[n][m]
}

/// Returns the vocabulary word a misspelled `word` most likely meant, if any.
/// Words shorter than five letters are never corrected; 5-7 letters allow one edit, 8+ allow two.
fn fuzzy_correct(word: &str, vocab: &[String]) -> Option<String> {
    if let Some((_, fix)) = TYPO_ALIASES.iter().find(|(typo, _)| *typo == word) {
        return Some((*fix).to_string());
    }
    let len = word.chars().count();
    if len < 5 || !word.chars().all(|c| c.is_alphabetic()) {
        return None;
    }
    if vocab.iter().any(|v| v == word) {
        return None;
    }
    let budget = if len >= 8 { 2 } else { 1 };
    let mut best: Option<(usize, &String)> = None;
    for v in vocab {
        let vlen = v.chars().count();
        if vlen < 5 || vlen.abs_diff(len) > budget {
            continue;
        }
        let d = edit_distance(word, v);
        if d <= budget && best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, v));
        }
    }
    best.map(|(_, v)| v.clone())
}

/// Splits `"title stuff // free-form notes"` at the first `//` that starts a word,
/// so URLs such as `https://…` stay in the title.
fn split_notes(input: &str) -> (&str, String) {
    for (idx, _) in input.match_indices("//") {
        let starts_word = idx == 0 || input[..idx].ends_with(char::is_whitespace);
        if starts_word {
            return (&input[..idx], input[idx + 2..].trim().to_string());
        }
    }
    (input, String::new())
}

/// Words that begin another directive and therefore end a `+step`.
fn is_directive(word: &str) -> bool {
    word == "*" || word == "+" || word == "^" || (word.len() > 1 && word.starts_with(['#', '@', '!', '~', '^', '+']))
}

/// Recognises an existing project written without `#`. Multi-word names are distinctive
/// enough to match anywhere; single-word names must be followed by "project". A leading
/// connector ("for", "in", "under", "into") is swallowed with the name.
/// Returns (project name, first word index, word count).
fn match_project_phrase(c: &Cursor, i: usize, known: &[String]) -> Option<(String, usize, usize)> {
    if c.consumed[i] {
        return None;
    }
    let max_len = 4.min(c.words.len() - i);
    for len in (1..=max_len).rev() {
        if (i..i + len).any(|k| c.consumed[k]) {
            continue;
        }
        let candidate = c.clean[i..i + len].join(" ");
        let Some(name) = known.iter().find(|p| p.to_lowercase() == candidate) else { continue };
        let followed_by_project = c.at(i + len) == Some("project");
        if !(followed_by_project || len >= 2) {
            continue;
        }
        let connector = i > 0 && !c.consumed[i - 1] && PROJECT_CONNECTORS.contains(&c.clean[i - 1].as_str());
        let start = if connector { i - 1 } else { i };
        let end = i + len + usize::from(followed_by_project);
        return Some((name.clone(), start, end - start));
    }
    None
}

pub fn format_minutes(m: u32) -> String {
    match (m / 60, m % 60) {
        (0, mins) => format!("{mins} min"),
        (h, 0) => format!("{h} h"),
        (h, mins) => format!("{h} h {mins} min"),
    }
}

fn weekday_from(s: &str) -> Option<u8> {
    let s = s.trim_end_matches('s'); // "mondays"
    if let Some(i) = WEEKDAYS.iter().position(|d| *d == s) {
        return Some(i as u8);
    }
    let long_forms = ["tues", "wednes", "thur", "thurs"];
    match s {
        "tues" => return Some(1),
        "wednes" => return Some(2),
        "thur" | "thurs" => return Some(3),
        _ => {}
    }
    let _ = long_forms;
    WEEKDAY_ABBR.iter().position(|d| *d == s).map(|i| i as u8)
}

fn is_weekday_abbreviation(s: &str) -> bool {
    WEEKDAY_ABBR.contains(&s)
}

fn month_from(s: &str) -> Option<u32> {
    if let Some(i) = MONTHS.iter().position(|m| *m == s) {
        return Some(i as u32 + 1);
    }
    if s == "sept" {
        return Some(9);
    }
    if s.len() == 3 {
        return MONTHS.iter().position(|m| m.starts_with(s)).map(|i| i as u32 + 1);
    }
    None
}

fn parse_day_number(s: &str) -> Option<u32> {
    let s = s.trim_end_matches("st").trim_end_matches("nd").trim_end_matches("rd").trim_end_matches("th");
    let d: u32 = s.parse().ok()?;
    (1..=31).contains(&d).then_some(d)
}

fn next_weekday(today: NaiveDate, target: u8, strictly_after: bool) -> NaiveDate {
    let cur = today.weekday().num_days_from_monday() as i64;
    let mut delta = (target as i64 - cur).rem_euclid(7);
    if delta == 0 && strictly_after {
        delta = 7;
    }
    today + Days::new(delta as u64)
}

/// The next occurrence of a day of the month: this month if it has not passed, else next.
fn next_day_of_month(today: NaiveDate, day: u32) -> Option<NaiveDate> {
    if let Some(d) = NaiveDate::from_ymd_opt(today.year(), today.month(), day) {
        if d >= today {
            return Some(d);
        }
    }
    let next = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)? + Months::new(1);
    NaiveDate::from_ymd_opt(next.year(), next.month(), day)
}

/// The first day of the next occurrence of a named month.
fn next_month_first(today: NaiveDate, month: u32) -> Option<NaiveDate> {
    let this = NaiveDate::from_ymd_opt(today.year(), month, 1)?;
    if this >= today { Some(this) } else { NaiveDate::from_ymd_opt(today.year() + 1, month, 1) }
}

/// Small spelled-out cardinals, for "in ten days" and "half past four".
fn number_word(s: &str) -> Option<u32> {
    const WORDS: [&str; 21] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen",
        "nineteen", "twenty",
    ];
    WORDS.iter().position(|w| *w == s).map(|i| i as u32)
}

/// Multi-word date phrases the single-word match cannot express: "the 20th",
/// "end of the month", "beginning of october", "the day after tomorrow", "a week today".
fn parse_date_phrase(c: &Cursor, i: usize, today: NaiveDate) -> Option<(NaiveDate, usize)> {
    let w = c.at(i)?;
    let next = c.at(i + 1);

    if w == "the" {
        // "the 20th" - the next time that day of the month comes round.
        if let Some(day) = next.and_then(parse_day_number) {
            if next.map(is_ordinal).unwrap_or(false) {
                return Some((next_day_of_month(today, day)?, 2));
            }
        }
        // "the day after tomorrow", "the day before friday"
        if next == Some("day") {
            match (c.at(i + 2), c.at(i + 3)) {
                (Some("after"), Some("tomorrow")) => return Some((today + Days::new(2), 4)),
                (Some("after"), Some("today")) => return Some((today + Days::new(1), 4)),
                (Some("before"), Some("tomorrow")) => return Some((today, 4)),
                (Some("before"), Some(d)) => {
                    if let Some(wd) = weekday_from(d) {
                        return Some((next_weekday(today, wd, false) - Days::new(1), 4));
                    }
                }
                _ => {}
            }
        }
        // "the month is out" (from "before the month is out")
        if next == Some("month") && c.at(i + 2) == Some("is") && c.at(i + 3) == Some("out") {
            return Some((end_of_month(today), 4));
        }
    }

    // A bare ordinal standing alone: "25th" with no month after it.
    if is_ordinal(w) && c.at(i + 1).and_then(month_from).is_none() {
        if let Some(day) = parse_day_number(w) {
            return Some((next_day_of_month(today, day)?, 1));
        }
    }

    // "end of the month", "start of october", "beginning of the week"
    if matches!(w, "end" | "start" | "beginning") {
        let mut k = i + 1;
        if c.at(k) == Some("of") {
            k += 1;
        }
        if c.at(k) == Some("the") {
            k += 1;
        }
        let unit = c.at(k)?;
        let len = k - i + 1;
        let is_end = w == "end";
        match unit {
            "month" => {
                let d = if is_end {
                    end_of_month(today)
                } else {
                    NaiveDate::from_ymd_opt(today.year(), today.month(), 1)? + Months::new(1)
                };
                return Some((d, len));
            }
            "week" => return Some((next_weekday(today, if is_end { 4 } else { 0 }, false), len)),
            "year" => {
                let d = if is_end {
                    NaiveDate::from_ymd_opt(today.year(), 12, 31)?
                } else {
                    NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)?
                };
                return Some((d, len));
            }
            other => {
                if let Some(m) = month_from(other) {
                    let first = next_month_first(today, m)?;
                    return Some((if is_end { end_of_month(first) } else { first }, len));
                }
            }
        }
    }

    // "a week today", "a fortnight from now"
    if matches!(w, "a" | "an" | "one") {
        let days: u64 = match next? {
            "week" => 7,
            "fortnight" => 14,
            _ => return None,
        };
        match (c.at(i + 2), c.at(i + 3)) {
            (Some("today"), _) => return Some((today + Days::new(days), 3)),
            (Some("from"), Some("now")) => return Some((today + Days::new(days), 4)),
            _ => {}
        }
    }
    None
}

fn is_ordinal(s: &str) -> bool {
    s.len() > 2 && (s.ends_with("st") || s.ends_with("nd") || s.ends_with("rd") || s.ends_with("th"))
}

fn end_of_month(d: NaiveDate) -> NaiveDate {
    let first_next = NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap_or(d) + Months::new(1);
    first_next.pred_opt().unwrap_or(d)
}

/// Returns (date, words consumed, implied time).
fn parse_date(c: &Cursor, i: usize, today: NaiveDate) -> Option<(NaiveDate, usize, Option<NaiveTime>)> {
    let w = c.at(i)?;

    // "before october" is the end of September, not October - handled before "before" is
    // swallowed as an ordinary deadline connector.
    if w == "before" {
        if let Some(m) = c.at(i + 1).and_then(month_from) {
            if let Some(first) = next_month_first(today, m) {
                return first.pred_opt().map(|d| (d, 2, None));
            }
        }
    }

    let mut prefix = 0;
    let mut j = i;
    if matches!(w, "on" | "by" | "due" | "before") {
        prefix = 1;
        j = i + 1;
    }
    if let Some((d, len)) = parse_date_phrase(c, j, today) {
        return Some((d, len + prefix, None));
    }
    let w = c.at(j)?;
    let result: Option<(NaiveDate, usize, Option<NaiveTime>)> = match w {
        "today" | "tod" | "eod" => Some((today, 1, None)),
        "tonight" => Some((today, 1, NaiveTime::from_hms_opt(20, 0, 0))),
        "tomorrow" | "tmr" | "tmrw" | "tom" => Some((today + Days::new(1), 1, None)),
        "yesterday" => Some((today - Days::new(1), 1, None)),
        "weekend" => Some((next_weekday(today, 5, false), 1, None)),
        "eow" => {
            let mut d = next_weekday(today, 4, false);
            if d < today {
                d = d + Days::new(7);
            }
            Some((d, 1, None))
        }
        "eom" => Some((end_of_month(today), 1, None)),
        "eoy" => Some((NaiveDate::from_ymd_opt(today.year(), 12, 31)?, 1, None)),
        "next" => match c.at(j + 1)? {
            "week" => Some((next_weekday(today, 0, true), 2, None)),
            "month" => Some((NaiveDate::from_ymd_opt(today.year(), today.month(), 1)? + Months::new(1), 2, None)),
            "year" => Some((NaiveDate::from_ymd_opt(today.year() + 1, 1, 1)?, 2, None)),
            "weekend" => Some((next_weekday(today, 5, true), 2, None)),
            other => weekday_from(other).map(|wd| (next_weekday(today, wd, true), 2, None)),
        },
        "this" => match c.at(j + 1)? {
            "weekend" => Some((next_weekday(today, 5, false), 2, None)),
            "week" => Some((next_weekday(today, 4, false), 2, None)),
            "month" => Some((end_of_month(today), 2, None)),
            other => weekday_from(other).map(|wd| (next_weekday(today, wd, false), 2, None)),
        },
        "in" => {
            let amount = c.at(j + 1)?;
            let qty: u32 = match amount {
                "a" | "an" => 1,
                other => number_word(other).or_else(|| other.parse().ok())?,
            };
            let unit = c.at(j + 2)?;
            let date = match unit {
                "day" | "days" | "d" => today + Days::new(qty as u64),
                "fortnight" | "fortnights" => today + Days::new(14 * qty as u64),
                "week" | "weeks" | "w" | "wk" | "wks" => today + Days::new(7 * qty as u64),
                "month" | "months" | "mo" | "mos" => today + Months::new(qty),
                "year" | "years" | "y" | "yr" | "yrs" => today + Months::new(12 * qty),
                _ => return None,
            };
            Some((date, 3, None))
        }
        _ => {
            // ISO date.
            if let Ok(d) = NaiveDate::parse_from_str(w, "%Y-%m-%d") {
                Some((d, 1, None))
            } else if let Some(d) = parse_slash_date(w, today) {
                Some((d, 1, None))
            } else if let Some(month) = month_from(w) {
                // "sep 15", "sep 15th", "sep 15 2026"
                let day = c.at(j + 1).and_then(parse_day_number)?;
                let (year, extra) = match c.at(j + 2).and_then(|y| y.parse::<i32>().ok()) {
                    Some(y) if (2000..=2100).contains(&y) => (y, 1),
                    _ => (today.year(), 0),
                };
                let mut d = NaiveDate::from_ymd_opt(year, month, day)?;
                if extra == 0 && d < today {
                    d = NaiveDate::from_ymd_opt(year + 1, month, day)?;
                }
                Some((d, 2 + extra, None))
            } else if let Some(day) = parse_day_number(w) {
                // "15 sep", "15th september"
                let month = c.at(j + 1).and_then(month_from)?;
                let (year, extra) = match c.at(j + 2).and_then(|y| y.parse::<i32>().ok()) {
                    Some(y) if (2000..=2100).contains(&y) => (y, 1),
                    _ => (today.year(), 0),
                };
                let mut d = NaiveDate::from_ymd_opt(year, month, day)?;
                if extra == 0 && d < today {
                    d = NaiveDate::from_ymd_opt(year + 1, month, day)?;
                }
                Some((d, 2 + extra, None))
            } else if let Some(wd) = weekday_from(w) {
                // Bare weekday. Abbreviations are only accepted when they are clearly a date
                // (prefixed, last word, or followed by a time) to keep "sun cream" a title.
                let last = j + 1 >= c.words.len();
                let followed_by_time = parse_time(c, j + 1).is_some();
                if is_weekday_abbreviation(w) && prefix == 0 && !last && !followed_by_time {
                    None
                } else {
                    Some((next_weekday(today, wd, false), 1, None))
                }
            } else {
                None
            }
        }
    };
    result.map(|(d, len, t)| (d, len + prefix, t))
}

fn parse_slash_date(w: &str, today: NaiveDate) -> Option<NaiveDate> {
    let parts: Vec<&str> = w.split('/').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let a: u32 = parts[0].parse().ok()?;
    let b: u32 = parts[1].parse().ok()?;
    let year: i32 = match parts.get(2) {
        Some(y) => {
            let y: i32 = y.parse().ok()?;
            if y < 100 { 2000 + y } else { y }
        }
        None => today.year(),
    };
    // m/d by default; d/m when the first number cannot be a month.
    let (month, day) = if a > 12 { (b, a) } else { (a, b) };
    let mut d = NaiveDate::from_ymd_opt(year, month, day)?;
    if parts.len() == 2 && d < today {
        d = NaiveDate::from_ymd_opt(year + 1, month, day)?;
    }
    Some(d)
}

/// Turns a bare clock hour into 24-hour form. People writing a to-do mean the afternoon
/// far more often than the small hours, so 1-8 read as PM and 9-12 as written.
fn bare_hour_to_24(h: u32) -> Option<u32> {
    match h {
        1..=8 => Some(h + 12),
        9..=12 => Some(h),
        0 | 13..=23 => Some(h),
        _ => None,
    }
}

/// Words that make a preceding number a duration, not a clock time. Without this,
/// "will take about 2 hours" has its "2" taken as 2pm before parse_duration ever sees it.
fn is_duration_unit(s: &str) -> bool {
    matches!(
        s,
        "h" | "hr" | "hrs" | "hour" | "hours"
            | "m" | "min" | "mins" | "minute" | "minutes"
            | "d" | "day" | "days" | "week" | "weeks"
    )
}

/// An hour word or digit: "seven", "7".
fn clock_hour(s: &str) -> Option<u32> {
    let h = number_word(s).or_else(|| s.parse::<u32>().ok())?;
    (0..=23).contains(&h).then_some(h)
}

/// "in the evening" / "in the morning" after an hour, which settles am vs pm.
/// Returns (hour in 24h, words consumed by the modifier).
fn period_modifier(c: &Cursor, i: usize, hour: u32) -> Option<(u32, usize)> {
    let mut k = i;
    if c.at(k) == Some("in") {
        k += 1;
    }
    if c.at(k) == Some("the") {
        k += 1;
    }
    let h12 = hour % 12;
    let adjusted = match c.at(k)? {
        "morning" => h12,
        "afternoon" | "evening" | "night" => h12 + 12,
        _ => return None,
    };
    Some((adjusted, k - i + 1))
}

/// Multi-word clock phrases: "quarter past two", "ten to six", "first thing", "after lunch".
fn parse_time_phrase(c: &Cursor, i: usize) -> Option<(NaiveTime, usize)> {
    let w = c.at(i)?;

    // Named points in the day. Only the widely understood ones: inventing a meaning for
    // every idiom ("knocking off time") would be fitting the benchmark, not the language.
    let named: Option<(u32, u32, usize)> = match (w, c.at(i + 1), c.at(i + 2)) {
        ("first", Some("thing"), _) => Some((9, 0, 2)),
        ("after", Some("lunch"), _) => Some((13, 0, 2)),
        ("before", Some("bed"), _) => Some((21, 0, 2)),
        ("close", Some("of"), Some("play")) => Some((17, 0, 3)),
        ("mid-morning", _, _) | ("midmorning", _, _) => Some((10, 0, 1)),
        ("mid-afternoon", _, _) => Some((15, 0, 1)),
        ("lunchtime", _, _) => Some((13, 0, 1)),
        ("teatime", _, _) => Some((17, 0, 1)),
        ("am", _, _) => Some((9, 0, 1)),
        ("pm", _, _) => Some((14, 0, 1)),
        _ => None,
    };
    if let Some((h, m, len)) = named {
        return NaiveTime::from_hms_opt(h, m, 0).map(|t| (t, len));
    }

    // "<minutes> past|to <hour>"
    let minutes = match w {
        "quarter" => Some(15),
        "half" => Some(30),
        other => number_word(other),
    }?;
    if minutes > 59 {
        return None;
    }
    let direction = c.at(i + 1)?;
    let hour = clock_hour(c.at(i + 2)?)?;
    let (h, m) = match direction {
        "past" => (hour, minutes),
        "to" => ((hour + 11) % 12, 60 - minutes),
        _ => return None,
    };
    let h = if m == 0 { h } else { h };
    let h24 = bare_hour_to_24(if h == 0 { 12 } else { h })?;
    let (h24, extra) = match period_modifier(c, i + 3, h24) {
        Some((adjusted, used)) => (adjusted, used),
        None => (h24, 0),
    };
    NaiveTime::from_hms_opt(h24, m, 0).map(|t| (t, 3 + extra))
}

/// Returns (time, words consumed).
fn parse_time(c: &Cursor, i: usize) -> Option<(NaiveTime, usize)> {
    let w = c.at(i)?;
    // Hedges people put in front of a clock time. Consuming them here is what lets
    // "around 5", "dead on 2" and "no later than 5" resolve at all.
    let prefix = match (w, c.at(i + 1), c.at(i + 2)) {
        ("no", Some("later"), Some("than")) => 3,
        ("just", Some("before"), _) | ("dead", Some("on"), _) => 2,
        ("at" | "around" | "about" | "approx" | "roughly" | "before" | "by", _, _) => 1,
        _ => 0,
    };
    // Tried at i first so a phrase that starts with a hedge word ("before bed",
    // "after lunch") is not decapitated by the hedge stripping.
    if let Some((t, len)) = parse_time_phrase(c, i) {
        return Some((t, len));
    }
    let j = i + prefix;
    if let Some((t, len)) = parse_time_phrase(c, j) {
        return Some((t, len + prefix));
    }
    let w = c.at(j)?;
    // "3ish" is the same as "3".
    let w = match w.strip_suffix("ish") {
        Some(base) if !base.is_empty() && base.chars().all(|ch| ch.is_ascii_digit()) => base,
        _ => w,
    };
    // A hedge in front of a duration ("about 2 hours") is not a clock time.
    if prefix > 0 && c.at(j + 1).map(is_duration_unit).unwrap_or(false) {
        return None;
    }
    let two_word_meridiem = matches!(c.at(j + 1), Some("am") | Some("pm"));

    let parsed: Option<(NaiveTime, usize)> = match w {
        "noon" | "midday" => NaiveTime::from_hms_opt(12, 0, 0).map(|t| (t, 1)),
        "midnight" => NaiveTime::from_hms_opt(0, 0, 0).map(|t| (t, 1)),
        "morning" => NaiveTime::from_hms_opt(9, 0, 0).map(|t| (t, 1)),
        "afternoon" => NaiveTime::from_hms_opt(14, 0, 0).map(|t| (t, 1)),
        "evening" => NaiveTime::from_hms_opt(18, 0, 0).map(|t| (t, 1)),
        _ => {
            let (body, meridiem, len) = if let Some(b) = w.strip_suffix("am") {
                (b, Some(false), 1)
            } else if let Some(b) = w.strip_suffix("pm") {
                (b, Some(true), 1)
            } else if two_word_meridiem {
                (w, Some(c.at(j + 1) == Some("pm")), 2)
            } else {
                (w, None, 1)
            };
            let (h, m) = match body.split_once(':') {
                Some((h, m)) => (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?),
                None => (number_word(body).or_else(|| body.parse::<u32>().ok())?, 0),
            };
            if m > 59 {
                return None;
            }
            match meridiem {
                Some(pm) => {
                    if !(1..=12).contains(&h) {
                        return None;
                    }
                    let hour = match (h, pm) {
                        (12, false) => 0,
                        (12, true) => 12,
                        (h, true) => h + 12,
                        (h, false) => h,
                    };
                    NaiveTime::from_hms_opt(hour, m, 0).map(|t| (t, len))
                }
                None => {
                    // Bare numbers only count as a time with a hedge prefix or a colon,
                    // so "call 3 people" stays a title.
                    if body.contains(':') && h <= 23 {
                        let (hour, extra) = match period_modifier(c, j + 1, h) {
                            Some((adjusted, used)) => (adjusted, used),
                            None => (h, 0),
                        };
                        NaiveTime::from_hms_opt(hour, m, 0).map(|t| (t, 1 + extra))
                    } else if prefix > 0 && (body.len() <= 2 || number_word(body).is_some()) {
                        // "at 9 30" - minutes written with a space instead of a colon.
                        let (minutes, mut used) = match c.at(j + 1).and_then(|n| n.parse::<u32>().ok()) {
                            Some(mm) if mm <= 59 && c.at(j + 1).map(|n| n.len()) == Some(2) => (mm, 2),
                            _ => (0, 1),
                        };
                        let hour = if minutes > 0 { h } else { bare_hour_to_24(h)? };
                        let (hour, extra) = match period_modifier(c, j + used, hour) {
                            Some((adjusted, e)) => (adjusted, e),
                            None => (hour, 0),
                        };
                        used += extra;
                        NaiveTime::from_hms_opt(hour, minutes, 0).map(|t| (t, used))
                    } else {
                        None
                    }
                }
            }
        }
    };
    parsed.map(|(t, len)| (t, len + prefix))
}

/// Returns (minutes, words consumed).
fn parse_duration(c: &Cursor, i: usize) -> Option<(u32, usize)> {
    let w = c.at(i)?;
    let (body, tilde) = match w.strip_prefix('~') {
        Some(b) => (b, true),
        None => (w, false),
    };
    if body.is_empty() {
        return None;
    }
    // Two-word form: "30 min", "2 hours", "1.5 hrs" (prefixed "for" is consumed too).
    if let Ok(num) = body.parse::<f64>() {
        let unit = c.at(i + 1)?;
        let minutes = match unit {
            "min" | "mins" | "minute" | "minutes" => num,
            "hour" | "hours" | "hr" | "hrs" => num * 60.0,
            _ => return None,
        };
        let m = minutes.round() as u32;
        return (m > 0).then_some((m, 2));
    }
    // Single-token forms: 45m, 45min, 2h, 1.5h, 1h30m, 1h30
    let s = body;
    if let Some((h, rest)) = s.split_once('h') {
        let hours: f64 = h.parse().ok()?;
        let rest = rest.trim_start_matches("rs").trim_start_matches('r').trim_start_matches("ours").trim_start_matches("our");
        let extra: u32 = if rest.is_empty() {
            0
        } else {
            rest.trim_end_matches("ins").trim_end_matches("in").trim_end_matches('m').parse().ok()?
        };
        let m = (hours * 60.0).round() as u32 + extra;
        return (m > 0).then_some((m, 1));
    }
    for suffix in ["minutes", "minute", "mins", "min", "m"] {
        if let Some(num) = s.strip_suffix(suffix) {
            let n: f64 = num.parse().ok()?;
            let m = n.round() as u32;
            // A bare "5m" is only an estimate when written with ~ or an explicit unit word.
            if suffix == "m" && !tilde && m < 5 {
                return None;
            }
            return (m > 0).then_some((m, 1));
        }
    }
    None
}

/// Returns (recurrence, words consumed, implied due date, implied time).
fn parse_recurrence(
    c: &Cursor,
    i: usize,
    today: NaiveDate,
) -> Option<(Recurrence, usize, Option<NaiveDate>, Option<NaiveTime>)> {
    let w = c.at(i)?;
    let rec = |kind, interval, weekday| Recurrence { kind, interval, weekday };
    match w {
        "daily" | "everyday" => Some((rec(RecurrenceKind::Daily, 1, None), 1, None, None)),
        "weekly" => Some((rec(RecurrenceKind::Weekly, 1, None), 1, None, None)),
        "fortnightly" | "biweekly" => Some((rec(RecurrenceKind::Weekly, 2, None), 1, None, None)),
        "monthly" => Some((rec(RecurrenceKind::Monthly, 1, None), 1, None, None)),
        "yearly" | "annually" => Some((rec(RecurrenceKind::Yearly, 1, None), 1, None, None)),
        "weekdays" => Some((rec(RecurrenceKind::Weekdays, 1, None), 1, None, None)),
        "every" | "each" => {
            let next = c.at(i + 1)?;
            match next {
                "day" => Some((rec(RecurrenceKind::Daily, 1, None), 2, None, None)),
                "morning" => Some((rec(RecurrenceKind::Daily, 1, None), 2, None, NaiveTime::from_hms_opt(9, 0, 0))),
                "evening" => Some((rec(RecurrenceKind::Daily, 1, None), 2, None, NaiveTime::from_hms_opt(18, 0, 0))),
                "night" => Some((rec(RecurrenceKind::Daily, 1, None), 2, None, NaiveTime::from_hms_opt(21, 0, 0))),
                "weekday" => Some((rec(RecurrenceKind::Weekdays, 1, None), 2, None, None)),
                "week" => Some((rec(RecurrenceKind::Weekly, 1, None), 2, None, None)),
                "month" => Some((rec(RecurrenceKind::Monthly, 1, None), 2, None, None)),
                "year" => Some((rec(RecurrenceKind::Yearly, 1, None), 2, None, None)),
                "other" => {
                    let unit = c.at(i + 2)?;
                    let kind = match unit {
                        "day" => RecurrenceKind::Daily,
                        "week" => RecurrenceKind::Weekly,
                        "month" => RecurrenceKind::Monthly,
                        "year" => RecurrenceKind::Yearly,
                        _ => return None,
                    };
                    Some((rec(kind, 2, None), 3, None, None))
                }
                _ => {
                    if let Some(wd) = weekday_from(next) {
                        let due = next_weekday(today, wd, false);
                        return Some((rec(RecurrenceKind::Weekly, 1, Some(wd)), 2, Some(due), None));
                    }
                    let n: u32 = next.parse().ok()?;
                    let unit = c.at(i + 2)?;
                    let kind = match unit {
                        "day" | "days" => RecurrenceKind::Daily,
                        "week" | "weeks" => RecurrenceKind::Weekly,
                        "month" | "months" => RecurrenceKind::Monthly,
                        "year" | "years" => RecurrenceKind::Yearly,
                        _ => return None,
                    };
                    Some((rec(kind, n.max(1), None), 3, None, None))
                }
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, 10).unwrap() // a Thursday
    }

    fn d(y: i32, m: u32, day: u32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m, day)
    }

    fn t(h: u32, m: u32) -> Option<NaiveTime> {
        NaiveTime::from_hms_opt(h, m, 0)
    }

    fn at(text: &str) -> Parsed {
        parse(text, today(), &[], &[])
    }

    #[test]
    fn resolves_day_of_month() {
        assert_eq!(at("settle the bill on the 20th").due_date, d(2026, 9, 20));
        // The 1st has passed this month, so it means next month.
        assert_eq!(at("pay the rent by the 1st").due_date, d(2026, 10, 1));
    }

    #[test]
    fn resolves_month_and_week_boundaries() {
        assert_eq!(at("file it end of the month").due_date, d(2026, 9, 30));
        assert_eq!(at("file it before the month is out").due_date, d(2026, 9, 30));
        assert_eq!(at("switch over beginning of october").due_date, d(2026, 10, 1));
        // "before <month>" is the last day of the month before it.
        assert_eq!(at("test the generator before october").due_date, d(2026, 9, 30));
        assert_eq!(at("submit the timesheet end of the week").due_date, d(2026, 9, 11));
    }

    #[test]
    fn resolves_relative_phrases() {
        assert_eq!(at("collect the car the day after tomorrow").due_date, d(2026, 9, 12));
        assert_eq!(at("chase the report a week today").due_date, d(2026, 9, 17));
        assert_eq!(at("review it a fortnight from now").due_date, d(2026, 9, 24));
        assert_eq!(at("renew the permit in ten days").due_date, d(2026, 9, 20));
    }

    #[test]
    fn resolves_spoken_clock_times() {
        assert_eq!(at("collect the glasses at quarter past two").due_time, t(14, 15));
        assert_eq!(at("return the kettle half past four").due_time, t(16, 30));
        assert_eq!(at("feed the fish ten to six").due_time, t(17, 50));
        assert_eq!(at("collect the kids quarter to four").due_time, t(15, 45));
        assert_eq!(at("phone the bank at twenty past nine").due_time, t(9, 20));
    }

    #[test]
    fn bare_hours_read_as_afternoon_unless_told_otherwise() {
        assert_eq!(at("ring the surveyor around 5").due_time, t(17, 0));
        assert_eq!(at("ping the supplier no later than 5").due_time, t(17, 0));
        assert_eq!(at("call them at 8").due_time, t(20, 0));
        // 9 to 12 are left as written.
        assert_eq!(at("submit the photo by 11am latest").due_time, t(11, 0));
        // An explicit period overrides the default.
        assert_eq!(at("reply to sir at 3 in the evening").due_time, t(15, 0));
        assert_eq!(at("start the cooker around seven in the morning").due_time, t(7, 0));
    }

    #[test]
    fn minutes_may_be_written_with_a_space() {
        let p = at("standup at 9 30");
        assert_eq!(p.due_time, t(9, 30));
        assert_eq!(p.title, "standup");
    }

    #[test]
    fn named_points_in_the_day() {
        assert_eq!(at("unlock the door first thing").due_time, t(9, 0));
        assert_eq!(at("call the roofer after lunch").due_time, t(13, 0));
        assert_eq!(at("stack the dishwasher before bed").due_time, t(21, 0));
        assert_eq!(at("close the till at close of play").due_time, t(17, 0));
    }

    #[test]
    fn hedged_durations_are_not_clock_times() {
        // "about 2" must not become 2pm and rob parse_duration of its number.
        let p = at("clean the garage, will take about 2 hours");
        assert_eq!(p.estimate_minutes, Some(120));
        assert_eq!(p.due_time, None);
    }

    #[test]
    fn bare_numbers_without_a_hedge_stay_in_the_title() {
        let p = at("ring 3 plumbers for quotes");
        assert_eq!(p.due_time, None);
        assert_eq!(p.title, "ring 3 plumbers for quotes");
        let q = at("print 12 copies of the agenda");
        assert_eq!(q.due_time, None);
        assert_eq!(q.due_date, None);
    }

    #[test]
    fn parses_everything_at_once() {
        let p = parse(
            "Send invoice to Acme tomorrow 5pm !p1 #Work @finance ~45m every friday",
            today(),
            &["Work".into()],
            &[],
        );
        assert_eq!(p.title, "Send invoice to Acme");
        assert_eq!(p.project.as_deref(), Some("Work"));
        assert_eq!(p.tags, vec!["finance".to_string()]);
        assert_eq!(p.priority, Some(Priority::P1));
        assert_eq!(p.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 11).unwrap()));
        assert_eq!(p.due_time, Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()));
        assert_eq!(p.estimate_minutes, Some(45));
        let rec = p.recurrence.unwrap();
        assert_eq!(rec.kind, RecurrenceKind::Weekly);
        assert_eq!(rec.weekday, Some(4));
    }

    #[test]
    fn multi_word_project_and_relative_dates() {
        let p = parse("Paint fence #Home Renovation in 2 weeks at 9:30", today(), &["Home Renovation".into()], &[]);
        assert_eq!(p.title, "Paint fence");
        assert_eq!(p.project.as_deref(), Some("Home Renovation"));
        assert_eq!(p.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 24).unwrap()));
        assert_eq!(p.due_time, Some(NaiveTime::from_hms_opt(9, 30, 0).unwrap()));
    }

    #[test]
    fn keeps_ambiguous_words_in_title() {
        let p = parse("Buy sun cream and read may report", today(), &[], &[]);
        assert_eq!(p.title, "Buy sun cream and read may report");
        assert!(p.due_date.is_none());
    }

    #[test]
    fn month_day_and_next_weekday() {
        let p = parse("Dentist on sep 15 at 2pm", today(), &[], &[]);
        assert_eq!(p.title, "Dentist");
        assert_eq!(p.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 15).unwrap()));
        assert_eq!(p.due_time, Some(NaiveTime::from_hms_opt(14, 0, 0).unwrap()));
        let p = parse("Team sync next monday", today(), &[], &[]);
        assert_eq!(p.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 14).unwrap()));
        let p = parse("Report by friday", today(), &[], &[]);
        assert_eq!(p.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 11).unwrap()));
        assert_eq!(p.title, "Report");
    }

    #[test]
    fn durations() {
        assert_eq!(parse("Deep work 1h30m", today(), &[], &[]).estimate_minutes, Some(90));
        assert_eq!(parse("Deep work 2 hours", today(), &[], &[]).estimate_minutes, Some(120));
        assert_eq!(parse("Deep work ~15m", today(), &[], &[]).estimate_minutes, Some(15));
        assert_eq!(parse("Swim 2m", today(), &[], &[]).estimate_minutes, None);
    }

    #[test]
    fn notes_steps_and_planned_date() {
        let p = parse("Plan launch ^tomorrow +outline +draft the post !p2 // remember the screenshots", today(), &[], &[]);
        assert_eq!(p.title, "Plan launch");
        assert_eq!(p.scheduled, Some(NaiveDate::from_ymd_opt(2026, 9, 11).unwrap()));
        assert_eq!(p.subtasks, vec!["outline".to_string(), "draft the post".to_string()]);
        assert_eq!(p.priority, Some(Priority::P2));
        assert_eq!(p.notes, "remember the screenshots");
        assert!(p.due_date.is_none());
        let p = parse("Read https://example.com/docs tomorrow", today(), &[], &[]);
        assert_eq!(p.title, "Read https://example.com/docs");
        assert!(p.notes.is_empty());
    }

    #[test]
    fn project_from_plain_text_and_estimate_phrases() {
        let known = vec!["Hacking Challenge".to_string(), "Workshop".to_string(), "Personal".to_string()];
        let p = parse("reply to sir at 3pm today in 10 mins, !p1, hacking challenge project", today(), &known, &[]);
        assert_eq!(p.title, "reply to sir");
        assert_eq!(p.project.as_deref(), Some("Hacking Challenge"));
        assert_eq!(p.estimate_minutes, Some(10));
        assert_eq!(p.priority, Some(Priority::P1));
        assert_eq!(p.due_date, Some(today()));
        assert_eq!(p.due_time, Some(NaiveTime::from_hms_opt(15, 0, 0).unwrap()));
        let p = parse("Fix projector for Hacking Challenge", today(), &known, &[]);
        assert_eq!(p.title, "Fix projector");
        assert_eq!(p.project.as_deref(), Some("Hacking Challenge"));
        let p = parse("Prep slides workshop project", today(), &known, &[]);
        assert_eq!(p.title, "Prep slides");
        assert_eq!(p.project.as_deref(), Some("Workshop"));
        // Single-word names stay ordinary words unless followed by "project" or written with #.
        let p = parse("Send personal update for about 1h", today(), &known, &[]);
        assert!(p.project.is_none());
        assert_eq!(p.title, "Send personal update");
        assert_eq!(p.estimate_minutes, Some(60));
    }

    #[test]
    fn tolerates_typos() {
        let known = vec!["Hacking Challenge".to_string()];
        let tags = vec!["finance".to_string()];
        let p = parse("reply to sir tommorow at 3pm !criticl #hacking chalenge", today(), &known, &tags);
        assert_eq!(p.title, "reply to sir");
        assert_eq!(p.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 11).unwrap()));
        assert_eq!(p.priority, Some(Priority::P1));
        assert_eq!(p.project.as_deref(), Some("Hacking Challenge"));
        let p = parse("pay rent @finnance evry month on fridy", today(), &known, &tags);
        assert_eq!(p.tags, vec!["finance".to_string()]);
        assert_eq!(p.recurrence.as_ref().map(|r| r.kind), Some(RecurrenceKind::Monthly));
        assert_eq!(p.due_date, Some(NaiveDate::from_ymd_opt(2026, 9, 11).unwrap()));
        assert_eq!(p.title, "pay rent");
        // Short and unrelated words are left alone.
        let p = parse("Fix the mouth guard and buy a mat", today(), &known, &tags);
        assert_eq!(p.title, "Fix the mouth guard and buy a mat");
        assert!(p.due_date.is_none());
        assert_eq!(edit_distance("tommorow", "tomorrow"), 2);
        assert_eq!(edit_distance("fridy", "friday"), 1);
    }

    #[test]
    fn recurrence_without_date_starts_today() {
        let p = parse("Stand-up every weekday 9am", today(), &[], &[]);
        assert_eq!(p.recurrence.unwrap().kind, RecurrenceKind::Weekdays);
        assert_eq!(p.due_date, Some(today()));
        assert_eq!(p.due_time, Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()));
        assert_eq!(p.title, "Stand-up");
    }
}
