//! HTTP API and static asset routes.

use crate::ai::{self, Ai};
use crate::analytics;
use crate::model::*;
use crate::parser::{self, Parsed};
use crate::priority::{self, DueStatus, Quadrant};
use crate::recurrence;
use crate::seed;
use crate::store::Store;
use axum::extract::{DefaultBodyLimit, FromRef, Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use chrono::{DateTime, Local, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;

pub type AppState = Arc<Store>;

/// Router state. Handlers extract `State<Arc<Store>>` or `State<Arc<Ai>>` as they need.
#[derive(Clone)]
pub struct App {
    pub store: Arc<Store>,
    pub ai: Arc<Ai>,
}

impl FromRef<App> for Arc<Store> {
    fn from_ref(app: &App) -> Self {
        app.store.clone()
    }
}

impl FromRef<App> for Arc<Ai> {
    fn from_ref(app: &App) -> Self {
        app.ai.clone()
    }
}

const INDEX_HTML: &str = include_str!("../static/index.html");
const APP_CSS: &str = include_str!("../static/app.css");
const APP_JS: &str = include_str!("../static/app.js");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found(what: &str) -> Self {
        ApiError { status: StatusCode::NOT_FOUND, message: format!("{what} not found") }
    }
    fn bad_request(msg: impl Into<String>) -> Self {
        ApiError { status: StatusCode::BAD_REQUEST, message: msg.into() }
    }
    fn internal(msg: impl Into<String>) -> Self {
        ApiError { status: StatusCode::INTERNAL_SERVER_ERROR, message: msg.into() }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        ApiError { status: StatusCode::INTERNAL_SERVER_ERROR, message: format!("storage error: {e}") }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(serde_json::json!({ "error": self.message }))).into_response()
    }
}

type ApiResult<T> = Result<Json<T>, ApiError>;

/// Distinguishes "field absent" from "field explicitly null" in PATCH bodies.
fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(de).map(Some)
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

struct Ctx {
    today: NaiveDate,
    now: DateTime<Utc>,
}

fn ctx() -> Ctx {
    Ctx { today: Local::now().date_naive(), now: Utc::now() }
}

#[derive(Serialize)]
pub struct TaskView {
    #[serde(flatten)]
    pub task: Task,
    pub score: u32,
    pub reasons: Vec<String>,
    pub urgent: bool,
    pub important: bool,
    pub quadrant: Quadrant,
    pub due_status: DueStatus,
    pub days_until_due: Option<i64>,
    pub recurrence_label: Option<String>,
    pub subtasks_done: usize,
    pub estimate_label: Option<String>,
}

fn to_view(task: &Task, db: &Database, ctx: &Ctx) -> TaskView {
    let s = priority::evaluate(task, ctx.today, ctx.now, &db.settings);
    TaskView {
        task: task.clone(),
        score: s.score,
        reasons: s.reasons,
        urgent: s.urgent,
        important: s.important,
        quadrant: s.quadrant,
        due_status: s.due_status,
        days_until_due: s.days_until_due,
        recurrence_label: task.recurrence.as_ref().map(|r| r.label()),
        subtasks_done: task.subtasks_done(),
        estimate_label: task.estimate_minutes.map(parser::format_minutes),
    }
}

fn view_of(store: &Store, id: &str) -> Result<TaskView, ApiError> {
    let db = store.read();
    let task = find_task(&db, id)?;
    Ok(to_view(task, &db, &ctx()))
}

fn find_task<'a>(db: &'a Database, id: &str) -> Result<&'a Task, ApiError> {
    db.tasks.iter().find(|t| t.id == id).ok_or_else(|| ApiError::not_found("task"))
}

fn find_task_mut<'a>(db: &'a mut Database, id: &str) -> Result<&'a mut Task, ApiError> {
    db.tasks.iter_mut().find(|t| t.id == id).ok_or_else(|| ApiError::not_found("task"))
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in tags {
        let t = t.trim().trim_start_matches(['#', '@']).to_lowercase();
        if !t.is_empty() && !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

#[derive(Serialize)]
struct TagCount {
    name: String,
    count: usize,
}

fn tag_counts(db: &Database) -> Vec<TagCount> {
    let mut map: BTreeMap<String, usize> = BTreeMap::new();
    for t in db.tasks.iter().filter(|t| t.is_open()) {
        for tag in &t.tags {
            *map.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    map.into_iter().map(|(name, count)| TagCount { name, count }).collect()
}

// ---------------------------------------------------------------------------
// Bootstrap / listing
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Bootstrap {
    version: &'static str,
    today: NaiveDate,
    now: DateTime<Utc>,
    settings: Settings,
    projects: Vec<Project>,
    tasks: Vec<TaskView>,
    tags: Vec<TagCount>,
    data_path: String,
}

async fn bootstrap(State(store): State<AppState>) -> ApiResult<Bootstrap> {
    let db = store.read();
    let c = ctx();
    let mut projects = db.projects.clone();
    projects.sort_by_key(|p| p.sort_order);
    Ok(Json(Bootstrap {
        version: env!("CARGO_PKG_VERSION"),
        today: c.today,
        now: c.now,
        settings: db.settings.clone(),
        projects,
        tasks: db.tasks.iter().map(|t| to_view(t, &db, &c)).collect(),
        tags: tag_counts(&db),
        data_path: store.path().display().to_string(),
    }))
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
}

async fn list_tasks(State(store): State<AppState>, Query(q): Query<ListQuery>) -> ApiResult<Vec<TaskView>> {
    let db = store.read();
    let c = ctx();
    let status = q.status.unwrap_or_else(|| "open".into());
    let tasks = db
        .tasks
        .iter()
        .filter(|t| match status.as_str() {
            "open" => t.is_open(),
            "done" => t.is_done(),
            "trash" => t.deleted_at.is_some(),
            _ => true,
        })
        .map(|t| to_view(t, &db, &c))
        .collect();
    Ok(Json(tasks))
}

async fn get_task(State(store): State<AppState>, Path(id): Path<String>) -> ApiResult<TaskView> {
    Ok(Json(view_of(&store, &id)?))
}

// ---------------------------------------------------------------------------
// Create / quick add / parse preview
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct CreateTask {
    title: String,
    #[serde(default)]
    notes: String,
    project_id: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    priority: Option<Priority>,
    due_date: Option<NaiveDate>,
    due_time: Option<NaiveTime>,
    scheduled: Option<NaiveDate>,
    estimate_minutes: Option<u32>,
    recurrence: Option<Recurrence>,
    #[serde(default)]
    starred: bool,
    #[serde(default)]
    subtasks: Vec<String>,
}

fn build_task(db: &Database, input: CreateTask, now: DateTime<Utc>) -> Result<Task, ApiError> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::bad_request("Task title cannot be empty"));
    }
    if let Some(pid) = &input.project_id {
        if !db.projects.iter().any(|p| &p.id == pid) {
            return Err(ApiError::not_found("project"));
        }
    }
    let sort_order = db.tasks.iter().map(|t| t.sort_order).max().unwrap_or(0) + 1;
    Ok(Task {
        id: new_id(),
        title,
        notes: input.notes,
        project_id: input.project_id,
        tags: normalize_tags(input.tags),
        priority: input.priority.unwrap_or_default(),
        due_date: input.due_date,
        due_time: input.due_time,
        scheduled: input.scheduled,
        estimate_minutes: input.estimate_minutes.filter(|m| *m > 0),
        time_spent_minutes: 0,
        subtasks: input
            .subtasks
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .map(|s| Subtask { id: new_id(), title: s.trim().to_string(), done: false })
            .collect(),
        recurrence: input.recurrence,
        starred: input.starred,
        completed_at: None,
        deleted_at: None,
        created_at: now,
        updated_at: now,
        sort_order,
    })
}

async fn create_task(State(store): State<AppState>, Json(input): Json<CreateTask>) -> Result<(StatusCode, Json<TaskView>), ApiError> {
    let id = store.write(|db| {
        let task = build_task(db, input, Utc::now())?;
        let id = task.id.clone();
        db.tasks.push(task);
        Ok::<_, ApiError>(id)
    })?;
    Ok((StatusCode::CREATED, Json(view_of(&store, &id)?)))
}

#[derive(Deserialize)]
struct QuickAdd {
    text: String,
    project_id: Option<String>,
    scheduled: Option<NaiveDate>,
    due_date: Option<NaiveDate>,
}

#[derive(Serialize)]
struct QuickAddResponse {
    task: TaskView,
    parsed: Parsed,
    created_project: Option<Project>,
}

fn project_names(db: &Database) -> Vec<String> {
    db.projects.iter().map(|p| p.name.clone()).collect()
}

fn tag_names(db: &Database) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for t in db.tasks.iter().filter(|t| t.deleted_at.is_none()) {
        for tag in &t.tags {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
    }
    tags
}

/// Parses text the way the add bar does and resolves the project against existing ones.
fn preview(db: &Database, text: &str, today: NaiveDate) -> ParsePreview {
    let parsed = parser::parse(text, today, &project_names(db), &tag_names(db));
    let project_id = parsed
        .project
        .as_ref()
        .and_then(|name| db.projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)))
        .map(|p| p.id.clone());
    let project_is_new = parsed.project.is_some() && project_id.is_none();
    ParsePreview { parsed, project_id, project_is_new }
}

fn find_or_create_project(db: &mut Database, name: &str, now: DateTime<Utc>) -> (String, Option<Project>) {
    if let Some(p) = db.projects.iter().find(|p| p.name.eq_ignore_ascii_case(name)) {
        return (p.id.clone(), None);
    }
    let color = seed::PROJECT_COLORS[db.projects.len() % seed::PROJECT_COLORS.len()].to_string();
    let project = Project {
        id: new_id(),
        name: name.to_string(),
        color,
        description: String::new(),
        created_at: now,
        sort_order: db.projects.iter().map(|p| p.sort_order).max().unwrap_or(0) + 1,
    };
    db.projects.push(project.clone());
    (project.id.clone(), Some(project))
}

async fn quick_add(State(store): State<AppState>, Json(input): Json<QuickAdd>) -> Result<(StatusCode, Json<QuickAddResponse>), ApiError> {
    let c = ctx();
    let (id, parsed, created_project) = store.write(|db| {
        let parsed = parser::parse(&input.text, c.today, &project_names(db), &tag_names(db));
        if parsed.title.is_empty() {
            return Err(ApiError::bad_request("Type a task title (the rest is optional)"));
        }
        let mut created_project = None;
        let project_id = match &parsed.project {
            Some(name) => {
                let (pid, created) = find_or_create_project(db, name, c.now);
                created_project = created;
                Some(pid)
            }
            None => input.project_id.clone(),
        };
        let task = build_task(
            db,
            CreateTask {
                title: parsed.title.clone(),
                project_id,
                tags: parsed.tags.clone(),
                priority: parsed.priority,
                notes: parsed.notes.clone(),
                due_date: parsed.due_date.or(input.due_date),
                due_time: parsed.due_time,
                scheduled: parsed.scheduled.or(input.scheduled),
                estimate_minutes: parsed.estimate_minutes,
                recurrence: parsed.recurrence.clone(),
                starred: parsed.starred,
                subtasks: parsed.subtasks.clone(),
                ..Default::default()
            },
            c.now,
        )?;
        let id = task.id.clone();
        db.tasks.push(task);
        Ok::<_, ApiError>((id, parsed, created_project))
    })?;
    Ok((StatusCode::CREATED, Json(QuickAddResponse { task: view_of(&store, &id)?, parsed, created_project })))
}

#[derive(Deserialize)]
struct ParseQuery {
    q: String,
}

#[derive(Serialize)]
struct ParsePreview {
    #[serde(flatten)]
    parsed: Parsed,
    project_id: Option<String>,
    project_is_new: bool,
}

async fn parse_preview(State(store): State<AppState>, Query(q): Query<ParseQuery>) -> ApiResult<ParsePreview> {
    let db = store.read();
    Ok(Json(preview(&db, &q.q, ctx().today)))
}

// ---------------------------------------------------------------------------
// Update / delete / lifecycle
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
struct UpdateTask {
    title: Option<String>,
    notes: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    project_id: Option<Option<String>>,
    tags: Option<Vec<String>>,
    priority: Option<Priority>,
    #[serde(default, deserialize_with = "double_option")]
    due_date: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "double_option")]
    due_time: Option<Option<NaiveTime>>,
    #[serde(default, deserialize_with = "double_option")]
    scheduled: Option<Option<NaiveDate>>,
    #[serde(default, deserialize_with = "double_option")]
    estimate_minutes: Option<Option<u32>>,
    #[serde(default, deserialize_with = "double_option")]
    recurrence: Option<Option<Recurrence>>,
    starred: Option<bool>,
    sort_order: Option<i64>,
    subtasks: Option<Vec<Subtask>>,
}

async fn update_task(State(store): State<AppState>, Path(id): Path<String>, Json(input): Json<UpdateTask>) -> ApiResult<TaskView> {
    store.write(|db| {
        if let Some(Some(pid)) = &input.project_id {
            if !db.projects.iter().any(|p| &p.id == pid) {
                return Err(ApiError::not_found("project"));
            }
        }
        let task = find_task_mut(db, &id)?;
        if let Some(title) = input.title {
            let title = title.trim().to_string();
            if title.is_empty() {
                return Err(ApiError::bad_request("Task title cannot be empty"));
            }
            task.title = title;
        }
        if let Some(notes) = input.notes {
            task.notes = notes;
        }
        if let Some(pid) = input.project_id {
            task.project_id = pid;
        }
        if let Some(tags) = input.tags {
            task.tags = normalize_tags(tags);
        }
        if let Some(p) = input.priority {
            task.priority = p;
        }
        if let Some(d) = input.due_date {
            task.due_date = d;
            if d.is_none() {
                task.due_time = None;
            }
        }
        if let Some(t) = input.due_time {
            task.due_time = t;
            if t.is_some() && task.due_date.is_none() {
                task.due_date = Some(ctx().today);
            }
        }
        if let Some(s) = input.scheduled {
            task.scheduled = s;
        }
        if let Some(e) = input.estimate_minutes {
            task.estimate_minutes = e.filter(|m| *m > 0);
        }
        if let Some(r) = input.recurrence {
            task.recurrence = r;
            if task.recurrence.is_some() && task.due_date.is_none() {
                task.due_date = Some(ctx().today);
            }
        }
        if let Some(s) = input.starred {
            task.starred = s;
        }
        if let Some(o) = input.sort_order {
            task.sort_order = o;
        }
        if let Some(subs) = input.subtasks {
            task.subtasks = subs
                .into_iter()
                .filter(|s| !s.title.trim().is_empty())
                .map(|mut s| {
                    if s.id.is_empty() {
                        s.id = new_id();
                    }
                    s.title = s.title.trim().to_string();
                    s
                })
                .collect();
        }
        task.updated_at = Utc::now();
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

async fn delete_task(State(store): State<AppState>, Path(id): Path<String>) -> ApiResult<TaskView> {
    store.write(|db| {
        let task = find_task_mut(db, &id)?;
        task.deleted_at = Some(Utc::now());
        task.updated_at = Utc::now();
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

async fn restore_task(State(store): State<AppState>, Path(id): Path<String>) -> ApiResult<TaskView> {
    store.write(|db| {
        let task = find_task_mut(db, &id)?;
        task.deleted_at = None;
        task.updated_at = Utc::now();
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

async fn purge_task(State(store): State<AppState>, Path(id): Path<String>) -> ApiResult<serde_json::Value> {
    store.write(|db| {
        let before = db.tasks.len();
        db.tasks.retain(|t| t.id != id);
        if db.tasks.len() == before {
            return Err(ApiError::not_found("task"));
        }
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Serialize)]
struct CompleteResponse {
    task: TaskView,
    next: Option<TaskView>,
}

/// Marks a task done. Recurring tasks spawn their next occurrence.
fn complete_in(db: &mut Database, id: &str, c: &Ctx) -> Result<Option<String>, ApiError> {
    let task = find_task_mut(db, id)?;
    if task.completed_at.is_some() {
        return Ok(None);
    }
    task.completed_at = Some(c.now);
    task.updated_at = c.now;
    let done = task.clone();
    if let Some(rec) = &done.recurrence {
        let from = done.due_date.unwrap_or(c.today);
        let next_due = recurrence::next_occurrence(rec, from, c.today);
        let mut next = done.clone();
        next.id = new_id();
        next.completed_at = None;
        next.due_date = Some(next_due);
        next.scheduled = None;
        next.time_spent_minutes = 0;
        next.created_at = c.now;
        next.updated_at = c.now;
        next.sort_order = db.tasks.iter().map(|t| t.sort_order).max().unwrap_or(0) + 1;
        for s in &mut next.subtasks {
            s.done = false;
            s.id = new_id();
        }
        let next_id = next.id.clone();
        db.tasks.push(next);
        return Ok(Some(next_id));
    }
    Ok(None)
}

async fn complete_task(State(store): State<AppState>, Path(id): Path<String>) -> ApiResult<CompleteResponse> {
    let c = ctx();
    let next_id = store.write(|db| complete_in(db, &id, &c))?;
    let next = match next_id {
        Some(nid) => Some(view_of(&store, &nid)?),
        None => None,
    };
    Ok(Json(CompleteResponse { task: view_of(&store, &id)?, next }))
}

async fn reopen_task(State(store): State<AppState>, Path(id): Path<String>) -> ApiResult<TaskView> {
    store.write(|db| {
        let task = find_task_mut(db, &id)?;
        task.completed_at = None;
        task.updated_at = Utc::now();
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

async fn duplicate_task(State(store): State<AppState>, Path(id): Path<String>) -> Result<(StatusCode, Json<TaskView>), ApiError> {
    let new_id_str = store.write(|db| {
        let src = find_task(db, &id)?.clone();
        let now = Utc::now();
        let mut copy = src;
        copy.id = new_id();
        copy.title = format!("{} (copy)", copy.title);
        copy.completed_at = None;
        copy.deleted_at = None;
        copy.time_spent_minutes = 0;
        copy.created_at = now;
        copy.updated_at = now;
        copy.sort_order = db.tasks.iter().map(|t| t.sort_order).max().unwrap_or(0) + 1;
        for s in &mut copy.subtasks {
            s.id = new_id();
            s.done = false;
        }
        let id = copy.id.clone();
        db.tasks.push(copy);
        Ok::<_, ApiError>(id)
    })?;
    Ok((StatusCode::CREATED, Json(view_of(&store, &new_id_str)?)))
}

#[derive(Deserialize)]
struct QuadrantRequest {
    quadrant: String,
}

#[derive(Serialize)]
struct QuadrantResponse {
    task: TaskView,
    note: Option<String>,
}

/// Moves a task in the Eisenhower matrix. Importance maps to priority; urgency maps to
/// "planned for today" so the change is reversible and never touches the deadline.
async fn set_quadrant(State(store): State<AppState>, Path(id): Path<String>, Json(input): Json<QuadrantRequest>) -> ApiResult<QuadrantResponse> {
    let c = ctx();
    let (want_important, want_urgent) = match input.quadrant.as_str() {
        "q1" => (true, true),
        "q2" => (true, false),
        "q3" => (false, true),
        "q4" => (false, false),
        _ => return Err(ApiError::bad_request("quadrant must be q1..q4")),
    };
    let note = store.write(|db| {
        let settings = db.settings.clone();
        let task = find_task_mut(db, &id)?;
        if want_important && !task.priority.is_important() {
            task.priority = Priority::P2;
        } else if !want_important && task.priority.is_important() {
            task.priority = Priority::P3;
        }
        let current = priority::evaluate(task, c.today, c.now, &settings);
        let mut note = None;
        if want_urgent && !current.urgent {
            task.scheduled = Some(c.today);
        } else if !want_urgent && current.urgent {
            task.scheduled = None;
            let after = priority::evaluate(task, c.today, c.now, &settings);
            if after.urgent {
                let due = match after.due_status {
                    DueStatus::Overdue => "overdue".to_string(),
                    DueStatus::Today => "due today".to_string(),
                    _ => format!("due in {} days", after.days_until_due.unwrap_or(0)),
                };
                note = Some(format!("Still urgent because it is {due}. Change the due date to move it."));
            }
        }
        task.updated_at = c.now;
        Ok::<_, ApiError>(note)
    })?;
    Ok(Json(QuadrantResponse { task: view_of(&store, &id)?, note }))
}

#[derive(Deserialize)]
struct LogTime {
    minutes: u32,
}

async fn log_time(State(store): State<AppState>, Path(id): Path<String>, Json(input): Json<LogTime>) -> ApiResult<TaskView> {
    if input.minutes == 0 {
        return Err(ApiError::bad_request("minutes must be positive"));
    }
    store.write(|db| {
        let now = Utc::now();
        let task = find_task_mut(db, &id)?;
        task.time_spent_minutes += input.minutes;
        task.updated_at = now;
        db.focus_sessions.push(FocusSession { id: new_id(), task_id: Some(id.clone()), minutes: input.minutes, ended_at: now });
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

// ---------------------------------------------------------------------------
// Subtasks
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SubtaskInput {
    title: String,
}

async fn add_subtask(State(store): State<AppState>, Path(id): Path<String>, Json(input): Json<SubtaskInput>) -> ApiResult<TaskView> {
    let title = input.title.trim().to_string();
    if title.is_empty() {
        return Err(ApiError::bad_request("Subtask title cannot be empty"));
    }
    store.write(|db| {
        let task = find_task_mut(db, &id)?;
        task.subtasks.push(Subtask { id: new_id(), title, done: false });
        task.updated_at = Utc::now();
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

#[derive(Deserialize)]
struct SubtaskUpdate {
    title: Option<String>,
    done: Option<bool>,
}

async fn update_subtask(State(store): State<AppState>, Path((id, sid)): Path<(String, String)>, Json(input): Json<SubtaskUpdate>) -> ApiResult<TaskView> {
    store.write(|db| {
        let task = find_task_mut(db, &id)?;
        let sub = task.subtasks.iter_mut().find(|s| s.id == sid).ok_or_else(|| ApiError::not_found("subtask"))?;
        if let Some(t) = input.title {
            let t = t.trim().to_string();
            if !t.is_empty() {
                sub.title = t;
            }
        }
        if let Some(d) = input.done {
            sub.done = d;
        }
        task.updated_at = Utc::now();
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

async fn delete_subtask(State(store): State<AppState>, Path((id, sid)): Path<(String, String)>) -> ApiResult<TaskView> {
    store.write(|db| {
        let task = find_task_mut(db, &id)?;
        task.subtasks.retain(|s| s.id != sid);
        task.updated_at = Utc::now();
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(view_of(&store, &id)?))
}

// ---------------------------------------------------------------------------
// Bulk actions & planning
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct BulkRequest {
    ids: Vec<String>,
    action: String,
    #[serde(default)]
    value: serde_json::Value,
}

#[derive(Serialize)]
struct BulkResponse {
    affected: usize,
}

async fn bulk(State(store): State<AppState>, Json(input): Json<BulkRequest>) -> ApiResult<BulkResponse> {
    let c = ctx();
    let affected = store.write(|db| {
        let mut affected = 0;
        if input.action == "purge" {
            let before = db.tasks.len();
            db.tasks.retain(|t| !input.ids.contains(&t.id));
            return Ok(before - db.tasks.len());
        }
        if input.action == "complete" {
            for id in &input.ids {
                if find_task(db, id).map(|t| t.completed_at.is_none()).unwrap_or(false) {
                    complete_in(db, id, &c)?;
                    affected += 1;
                }
            }
            return Ok(affected);
        }
        let value = input.value.clone();
        for id in &input.ids {
            let Ok(task) = find_task_mut(db, id) else { continue };
            match input.action.as_str() {
                "reopen" => task.completed_at = None,
                "delete" => task.deleted_at = Some(c.now),
                "restore" => task.deleted_at = None,
                "priority" => {
                    let p = value.as_str().and_then(Priority::parse_loose).ok_or_else(|| ApiError::bad_request("invalid priority"))?;
                    task.priority = p;
                }
                "project" => task.project_id = value.as_str().map(|s| s.to_string()),
                "schedule" => task.scheduled = value.as_str().and_then(|s| s.parse().ok()),
                "due" => {
                    task.due_date = value.as_str().and_then(|s| s.parse().ok());
                    if task.due_date.is_none() {
                        task.due_time = None;
                    }
                }
                "star" => task.starred = value.as_bool().unwrap_or(true),
                "tag" => {
                    if let Some(tag) = value.as_str() {
                        let mut tags = task.tags.clone();
                        tags.push(tag.to_string());
                        task.tags = normalize_tags(tags);
                    }
                }
                _ => return Err(ApiError::bad_request("unknown bulk action")),
            }
            task.updated_at = c.now;
            affected += 1;
        }
        Ok::<_, ApiError>(affected)
    })?;
    Ok(Json(BulkResponse { affected }))
}

#[derive(Deserialize, Default)]
struct PlanDayRequest {
    capacity_minutes: Option<u32>,
}

#[derive(Serialize)]
struct PlanDayResponse {
    planned: Vec<TaskView>,
    planned_minutes: u32,
    already_planned_minutes: u32,
    capacity_minutes: u32,
    skipped_for_capacity: usize,
}

/// Fills today with the highest-scoring open tasks until the daily capacity is used up.
/// Anything already due today or overdue is always included.
async fn plan_day(State(store): State<AppState>, Json(input): Json<PlanDayRequest>) -> ApiResult<PlanDayResponse> {
    let c = ctx();
    let (ids, planned_minutes, already, capacity, skipped) = store.write(|db| {
        let settings = db.settings.clone();
        let capacity = input.capacity_minutes.unwrap_or(settings.daily_capacity_minutes);
        let default_est = settings.default_estimate_minutes.max(5);
        let est = |t: &Task| t.estimate_minutes.unwrap_or(default_est);

        let already: u32 = db
            .tasks
            .iter()
            .filter(|t| t.is_open() && t.scheduled.map(|s| s <= c.today).unwrap_or(false))
            .map(est)
            .sum();

        let mut candidates: Vec<(u32, i64, String, u32, bool)> = db
            .tasks
            .iter()
            .filter(|t| t.is_open() && !t.scheduled.map(|s| s <= c.today).unwrap_or(false))
            .map(|t| {
                let s = priority::evaluate(t, c.today, c.now, &settings);
                let must = matches!(s.due_status, DueStatus::Overdue | DueStatus::Today);
                (s.score, t.due_date.map(|d| (d - c.today).num_days()).unwrap_or(i64::MAX), t.id.clone(), est(t), must)
            })
            .collect();
        candidates.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

        let mut remaining = capacity.saturating_sub(already) as i64;
        let mut ids = Vec::new();
        let mut planned_minutes = 0;
        let mut skipped = 0;
        for (_, _, id, minutes, must) in candidates {
            if must || minutes as i64 <= remaining {
                remaining -= minutes as i64;
                planned_minutes += minutes;
                ids.push(id);
            } else {
                skipped += 1;
            }
        }
        for id in &ids {
            if let Ok(t) = find_task_mut(db, id) {
                t.scheduled = Some(c.today);
                t.updated_at = c.now;
            }
        }
        Ok::<_, ApiError>((ids, planned_minutes, already, capacity, skipped))
    })?;
    let mut planned = Vec::new();
    for id in ids {
        planned.push(view_of(&store, &id)?);
    }
    Ok(Json(PlanDayResponse {
        planned,
        planned_minutes,
        already_planned_minutes: already,
        capacity_minutes: capacity,
        skipped_for_capacity: skipped,
    }))
}

// ---------------------------------------------------------------------------
// Search & analytics
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search(State(store): State<AppState>, Query(q): Query<SearchQuery>) -> ApiResult<Vec<TaskView>> {
    let needle = q.q.trim().to_lowercase();
    let db = store.read();
    let c = ctx();
    if needle.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let mut hits: Vec<(i32, TaskView)> = db
        .tasks
        .iter()
        .filter(|t| t.deleted_at.is_none())
        .filter_map(|t| {
            let title = t.title.to_lowercase();
            let mut rank = 0;
            if title.starts_with(&needle) {
                rank += 6;
            } else if title.contains(&needle) {
                rank += 4;
            }
            if t.tags.iter().any(|tag| tag.contains(&needle)) {
                rank += 3;
            }
            if t.notes.to_lowercase().contains(&needle) {
                rank += 1;
            }
            if t.subtasks.iter().any(|s| s.title.to_lowercase().contains(&needle)) {
                rank += 1;
            }
            if t.completed_at.is_some() {
                rank -= 2;
            }
            (rank > 0 || (rank == 0 && false)).then(|| (rank, to_view(t, &db, &c)))
        })
        .collect();
    hits.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.score.cmp(&a.1.score)));
    Ok(Json(hits.into_iter().take(40).map(|(_, v)| v).collect()))
}

async fn get_analytics(State(store): State<AppState>) -> ApiResult<analytics::Analytics> {
    let db = store.read();
    Ok(Json(analytics::compute(&db, ctx().today)))
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ProjectInput {
    name: String,
    color: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct ProjectUpdate {
    name: Option<String>,
    color: Option<String>,
    description: Option<String>,
    sort_order: Option<i64>,
}

fn valid_color(c: &str) -> bool {
    c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|ch| ch.is_ascii_hexdigit())
}

async fn list_projects(State(store): State<AppState>) -> ApiResult<Vec<Project>> {
    let db = store.read();
    let mut projects = db.projects.clone();
    projects.sort_by_key(|p| p.sort_order);
    Ok(Json(projects))
}

async fn create_project(State(store): State<AppState>, Json(input): Json<ProjectInput>) -> Result<(StatusCode, Json<Project>), ApiError> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(ApiError::bad_request("Project name cannot be empty"));
    }
    let project = store.write(|db| {
        if db.projects.iter().any(|p| p.name.eq_ignore_ascii_case(&name)) {
            return Err(ApiError::bad_request("A project with that name already exists"));
        }
        let color = input
            .color
            .filter(|c| valid_color(c))
            .unwrap_or_else(|| seed::PROJECT_COLORS[db.projects.len() % seed::PROJECT_COLORS.len()].to_string());
        let project = Project {
            id: new_id(),
            name,
            color,
            description: input.description.unwrap_or_default(),
            created_at: Utc::now(),
            sort_order: db.projects.iter().map(|p| p.sort_order).max().unwrap_or(0) + 1,
        };
        db.projects.push(project.clone());
        Ok(project)
    })?;
    Ok((StatusCode::CREATED, Json(project)))
}

async fn update_project(State(store): State<AppState>, Path(id): Path<String>, Json(input): Json<ProjectUpdate>) -> ApiResult<Project> {
    let project = store.write(|db| {
        if let Some(name) = &input.name {
            if db.projects.iter().any(|p| p.id != id && p.name.eq_ignore_ascii_case(name.trim())) {
                return Err(ApiError::bad_request("A project with that name already exists"));
            }
        }
        let project = db.projects.iter_mut().find(|p| p.id == id).ok_or_else(|| ApiError::not_found("project"))?;
        if let Some(name) = input.name {
            let name = name.trim().to_string();
            if name.is_empty() {
                return Err(ApiError::bad_request("Project name cannot be empty"));
            }
            project.name = name;
        }
        if let Some(color) = input.color {
            if !valid_color(&color) {
                return Err(ApiError::bad_request("Color must look like #rrggbb"));
            }
            project.color = color;
        }
        if let Some(d) = input.description {
            project.description = d;
        }
        if let Some(o) = input.sort_order {
            project.sort_order = o;
        }
        Ok(project.clone())
    })?;
    Ok(Json(project))
}

#[derive(Deserialize)]
struct DeleteProjectQuery {
    mode: Option<String>,
}

async fn delete_project(State(store): State<AppState>, Path(id): Path<String>, Query(q): Query<DeleteProjectQuery>) -> ApiResult<serde_json::Value> {
    let delete_tasks = q.mode.as_deref() == Some("delete");
    store.write(|db| {
        let before = db.projects.len();
        db.projects.retain(|p| p.id != id);
        if db.projects.len() == before {
            return Err(ApiError::not_found("project"));
        }
        let now = Utc::now();
        for t in db.tasks.iter_mut().filter(|t| t.project_id.as_deref() == Some(id.as_str())) {
            t.project_id = None;
            if delete_tasks && t.deleted_at.is_none() {
                t.deleted_at = Some(now);
            }
            t.updated_at = now;
        }
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---------------------------------------------------------------------------
// Settings, export/import, maintenance
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SettingsUpdate {
    user_name: Option<String>,
    workspace_name: Option<String>,
    daily_capacity_minutes: Option<u32>,
    urgent_window_days: Option<i64>,
    pomodoro_minutes: Option<u32>,
    default_estimate_minutes: Option<u32>,
    theme: Option<String>,
    dark_style: Option<String>,
    accent: Option<String>,
    ai_enabled: Option<bool>,
    ai_model: Option<String>,
    ai_auto: Option<bool>,
}

async fn get_settings(State(store): State<AppState>) -> ApiResult<Settings> {
    Ok(Json(store.read().settings.clone()))
}

async fn update_settings(State(store): State<AppState>, Json(input): Json<SettingsUpdate>) -> ApiResult<Settings> {
    let settings = store.write(|db| {
        let s = &mut db.settings;
        if let Some(v) = input.user_name {
            s.user_name = v.trim().to_string();
        }
        if let Some(v) = input.workspace_name {
            s.workspace_name = v.trim().to_string();
        }
        if let Some(v) = input.daily_capacity_minutes {
            s.daily_capacity_minutes = v.clamp(30, 24 * 60);
        }
        if let Some(v) = input.urgent_window_days {
            s.urgent_window_days = v.clamp(0, 30);
        }
        if let Some(v) = input.pomodoro_minutes {
            s.pomodoro_minutes = v.clamp(1, 180);
        }
        if let Some(v) = input.default_estimate_minutes {
            s.default_estimate_minutes = v.clamp(5, 480);
        }
        if let Some(v) = input.theme {
            if matches!(v.as_str(), "system" | "light" | "dark") {
                s.theme = v;
            }
        }
        if let Some(v) = input.dark_style {
            if !DARK_STYLES.contains(&v.as_str()) {
                return Err(ApiError::bad_request("dark_style must be soft or plain"));
            }
            s.dark_style = v;
        }
        if let Some(v) = input.accent {
            let v = v.trim().to_lowercase();
            if !(ACCENTS.contains(&v.as_str()) || valid_color(&v)) {
                return Err(ApiError::bad_request(format!(
                    "accent must be a #rrggbb colour or one of {}",
                    ACCENTS.join(", ")
                )));
            }
            s.accent = v;
        }
        if let Some(v) = input.ai_enabled {
            s.ai_enabled = v;
        }
        if let Some(v) = input.ai_model {
            if ai::spec(&v).is_none() {
                return Err(ApiError::bad_request(format!("unknown model '{v}'")));
            }
            s.ai_model = v;
        }
        if let Some(v) = input.ai_auto {
            s.ai_auto = v;
        }
        Ok::<_, ApiError>(s.clone())
    })?;
    Ok(Json(settings))
}

async fn export(State(store): State<AppState>) -> Response {
    let db = store.read().clone();
    let body = serde_json::to_string_pretty(&db).unwrap_or_else(|_| "{}".into());
    let filename = format!("ordo-export-{}.json", Local::now().format("%Y-%m-%d"));
    (
        [
            (header::CONTENT_TYPE, "application/json; charset=utf-8".to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{filename}\"")),
        ],
        body,
    )
        .into_response()
}

#[derive(Deserialize)]
struct ImportQuery {
    mode: Option<String>,
}

#[derive(Serialize)]
struct ImportResponse {
    tasks: usize,
    projects: usize,
    mode: String,
}

async fn import(State(store): State<AppState>, Query(q): Query<ImportQuery>, Json(incoming): Json<Database>) -> ApiResult<ImportResponse> {
    let mode = q.mode.unwrap_or_else(|| "merge".into());
    let (tasks, projects) = store.write(|db| {
        if mode == "replace" {
            *db = incoming;
        } else {
            for p in incoming.projects {
                match db.projects.iter_mut().find(|x| x.id == p.id) {
                    Some(existing) => *existing = p,
                    None => db.projects.push(p),
                }
            }
            for t in incoming.tasks {
                match db.tasks.iter_mut().find(|x| x.id == t.id) {
                    Some(existing) => *existing = t,
                    None => db.tasks.push(t),
                }
            }
            for s in incoming.focus_sessions {
                if !db.focus_sessions.iter().any(|x| x.id == s.id) {
                    db.focus_sessions.push(s);
                }
            }
        }
        Ok::<_, ApiError>((db.tasks.len(), db.projects.len()))
    })?;
    Ok(Json(ImportResponse { tasks, projects, mode }))
}

async fn empty_trash(State(store): State<AppState>) -> ApiResult<BulkResponse> {
    let affected = store.write(|db| {
        let before = db.tasks.len();
        db.tasks.retain(|t| t.deleted_at.is_none());
        Ok::<_, ApiError>(before - db.tasks.len())
    })?;
    Ok(Json(BulkResponse { affected }))
}

async fn reset_demo(State(store): State<AppState>) -> ApiResult<serde_json::Value> {
    store.write(|db| {
        let settings = db.settings.clone();
        *db = seed::demo_database(ctx().today);
        db.settings = settings;
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn clear_all(State(store): State<AppState>) -> ApiResult<serde_json::Value> {
    store.write(|db| {
        let settings = db.settings.clone();
        *db = Database::default();
        db.settings = settings;
        Ok::<_, ApiError>(())
    })?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---------------------------------------------------------------------------
// On-device AI
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct AiModelInfo {
    #[serde(flatten)]
    spec: &'static ai::ModelSpec,
    status: ai::Status,
}

#[derive(Serialize)]
struct AiStatusResponse {
    enabled: bool,
    auto: bool,
    model: String,
    dir: String,
    models: Vec<AiModelInfo>,
}

async fn ai_status(State(store): State<AppState>, State(ai): State<Arc<Ai>>) -> ApiResult<AiStatusResponse> {
    let (enabled, auto, model) = {
        let db = store.read();
        (db.settings.ai_enabled, db.settings.ai_auto, db.settings.ai_model.clone())
    };
    let models = ai::MODELS.iter().map(|spec| AiModelInfo { spec, status: ai.status(spec) }).collect();
    Ok(Json(AiStatusResponse { enabled, auto, model, dir: ai.dir().display().to_string(), models }))
}

#[derive(Deserialize)]
struct AiModelRequest {
    model: String,
}

fn ai_spec(id: &str) -> Result<&'static ai::ModelSpec, ApiError> {
    ai::spec(id).ok_or_else(|| ApiError::bad_request(format!("unknown model '{id}'")))
}

async fn ai_download(State(ai): State<Arc<Ai>>, Json(input): Json<AiModelRequest>) -> ApiResult<serde_json::Value> {
    let spec = ai_spec(&input.model)?;
    ai.start_download(spec).map_err(ApiError::internal)?;
    Ok(Json(serde_json::json!({ "ok": true, "status": ai.status(spec) })))
}

async fn ai_remove(State(ai): State<Arc<Ai>>, Json(input): Json<AiModelRequest>) -> ApiResult<serde_json::Value> {
    let spec = ai_spec(&input.model)?;
    ai.remove(spec).map_err(ApiError::internal)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn ai_unload(State(ai): State<Arc<Ai>>) -> ApiResult<serde_json::Value> {
    ai.unload();
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
struct AiNormalizeRequest {
    text: String,
}

#[derive(Serialize)]
struct AiNormalizeResponse {
    #[serde(flatten)]
    result: ai::Normalized,
    parsed: ParsePreview,
}

/// Rewrites free text into quick-add syntax with the on-device model, then parses it.
async fn ai_normalize(
    State(store): State<AppState>,
    State(ai): State<Arc<Ai>>,
    Json(input): Json<AiNormalizeRequest>,
) -> ApiResult<AiNormalizeResponse> {
    let text = input.text.trim().to_string();
    if text.is_empty() {
        return Err(ApiError::bad_request("Type something first"));
    }
    let (model_id, projects) = {
        let db = store.read();
        (db.settings.ai_model.clone(), project_names(&db))
    };
    let spec = ai_spec(&model_id)?;
    let today = ctx().today;
    let worker = ai.clone();
    let result = tokio::task::spawn_blocking(move || worker.normalize(spec, &text, today, &projects))
        .await
        .map_err(|e| ApiError::internal(format!("AI worker failed: {e}")))?
        .map_err(ApiError::bad_request)?;
    let parsed = preview(&store.read(), &result.output, today);
    Ok(Json(AiNormalizeResponse { result, parsed }))
}

// ---------------------------------------------------------------------------
// Static assets
// ---------------------------------------------------------------------------

fn asset(content_type: &'static str, body: &'static str) -> Response {
    ([(header::CONTENT_TYPE, content_type), (header::CACHE_CONTROL, "no-cache")], body).into_response()
}

async fn index() -> Response {
    asset("text/html; charset=utf-8", INDEX_HTML)
}

async fn css() -> Response {
    asset("text/css; charset=utf-8", APP_CSS)
}

async fn js() -> Response {
    asset("application/javascript; charset=utf-8", APP_JS)
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router(store: Arc<Store>, ai: Arc<Ai>) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/app.css", get(css))
        .route("/app.js", get(js))
        .route("/api/bootstrap", get(bootstrap))
        .route("/api/parse", get(parse_preview))
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route("/api/tasks/quick", post(quick_add))
        .route("/api/tasks/bulk", post(bulk))
        .route("/api/tasks/{id}", get(get_task).patch(update_task).delete(delete_task))
        .route("/api/tasks/{id}/complete", post(complete_task))
        .route("/api/tasks/{id}/reopen", post(reopen_task))
        .route("/api/tasks/{id}/restore", post(restore_task))
        .route("/api/tasks/{id}/purge", delete(purge_task))
        .route("/api/tasks/{id}/duplicate", post(duplicate_task))
        .route("/api/tasks/{id}/quadrant", post(set_quadrant))
        .route("/api/tasks/{id}/time", post(log_time))
        .route("/api/tasks/{id}/subtasks", post(add_subtask))
        .route("/api/tasks/{id}/subtasks/{sid}", patch(update_subtask).delete(delete_subtask))
        .route("/api/plan-day", post(plan_day))
        .route("/api/search", get(search))
        .route("/api/analytics", get(get_analytics))
        .route("/api/projects", get(list_projects).post(create_project))
        .route("/api/projects/{id}", patch(update_project).delete(delete_project))
        .route("/api/settings", get(get_settings).patch(update_settings))
        .route("/api/export", get(export))
        .route("/api/import", post(import))
        .route("/api/trash/empty", post(empty_trash))
        .route("/api/reset-demo", post(reset_demo))
        .route("/api/clear-all", post(clear_all))
        .route("/api/ai/status", get(ai_status))
        .route("/api/ai/download", post(ai_download))
        .route("/api/ai/remove", post(ai_remove))
        .route("/api/ai/unload", post(ai_unload))
        .route("/api/ai/normalize", post(ai_normalize))
        .fallback(get(index))
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024))
        .with_state(App { store, ai })
}
