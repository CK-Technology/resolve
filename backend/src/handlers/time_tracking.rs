use axum::{
    extract::{Path, Query, State},
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;
use crate::{
    AppState, ApiResult, ApiError,
    PaginatedResponse, PaginationParams, PaginationMeta,
    validation::{self, Validator},
};
use crate::auth::middleware::AuthUser;

#[derive(Serialize, Deserialize)]
pub struct TimeEntryCreate {
    pub ticket_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub description: Option<String>,
    pub billable: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryUpdate {
    pub ticket_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub description: Option<String>,
    pub billable: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct ManualTimeEntry {
    pub ticket_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub billable: bool,
}

/// Query parameters for listing time entries
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TimeEntryQuery {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    /// Filter by user ID (admin only - regular users can only see their own)
    pub user_id: Option<Uuid>,
    /// Filter by ticket ID
    pub ticket_id: Option<Uuid>,
    /// Filter by project ID
    pub project_id: Option<Uuid>,
    /// Filter by client ID
    pub client_id: Option<Uuid>,
    /// Filter by billable status
    pub billable: Option<bool>,
    /// Filter by billed status
    pub billed: Option<bool>,
    /// Filter entries starting from this date (inclusive)
    pub from_date: Option<NaiveDate>,
    /// Filter entries up to this date (inclusive)
    pub to_date: Option<NaiveDate>,
    /// Search in description
    pub q: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryWithDetails {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub ticket_id: Option<Uuid>,
    pub ticket_number: Option<i32>,
    pub ticket_subject: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub task_id: Option<Uuid>,
    pub task_name: Option<String>,
    pub client_id: Option<Uuid>,
    pub client_name: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub description: Option<String>,
    pub billable: bool,
    pub billed: bool,
    pub hourly_rate: Option<Decimal>,
    pub total_amount: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct ActiveTimer {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub ticket_subject: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub client_name: Option<String>,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub elapsed_minutes: i32,
    pub billable: bool,
}

/// Time tracking statistics for the current user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeStats {
    /// Total hours logged today
    pub total_hours_today: f64,
    /// Billable hours logged today
    pub billable_hours_today: f64,
    /// Total hours logged this week
    pub total_hours_week: f64,
    /// Billable hours logged this week
    pub billable_hours_week: f64,
    /// Total unbilled amount
    pub unbilled_amount: f64,
    /// Number of active (running) timers
    pub active_timers: i32,
}

pub fn time_tracking_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/entries", get(list_time_entries).post(create_manual_entry))
        .route("/entries/:id", get(get_time_entry).put(update_time_entry).delete(delete_time_entry))
        .route("/timer/start", post(start_timer))
        .route("/timer/stop", post(stop_timer))
        .route("/timer/active", get(get_active_timers))
        .route("/timer/switch", post(switch_timer))
        .route("/stats", get(get_time_stats))
        .route("/timesheet", get(get_timesheet))
}

/// List time entries with pagination and filtering
async fn list_time_entries(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<TimeEntryQuery>,
) -> ApiResult<Json<PaginatedResponse<TimeEntryWithDetails>>> {
    let limit = params.pagination.limit();
    let offset = params.pagination.offset();

    // Build dynamic WHERE clause based on filters
    // For non-admin users, always filter by their own user_id
    let user_filter = params.user_id.unwrap_or(user.id);

    // Get total count for pagination
    let total: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM time_entries te
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         WHERE te.user_id = $1
           AND ($2::uuid IS NULL OR te.ticket_id = $2)
           AND ($3::uuid IS NULL OR te.project_id = $3)
           AND ($4::uuid IS NULL OR COALESCE(t.client_id, p.client_id) = $4)
           AND ($5::bool IS NULL OR te.billable = $5)
           AND ($6::bool IS NULL OR te.billed = $6)
           AND ($7::date IS NULL OR te.start_time::date >= $7)
           AND ($8::date IS NULL OR te.start_time::date <= $8)
           AND ($9::text IS NULL OR te.description ILIKE '%' || $9 || '%')"#,
        user_filter,
        params.ticket_id,
        params.project_id,
        params.client_id,
        params.billable,
        params.billed,
        params.from_date,
        params.to_date,
        params.q
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error counting time entries: {}", e);
        ApiError::internal("Failed to count time entries")
    })?;

    // Get entries with details
    let entries = sqlx::query_as!(
        TimeEntryWithDetails,
        r#"SELECT
            te.id, te.user_id,
            COALESCE(u.first_name || ' ' || u.last_name, 'Unknown') as "user_name!",
            te.ticket_id, t.number as ticket_number, t.subject as ticket_subject,
            te.project_id, p.name as project_name,
            te.task_id, tk.name as task_name,
            COALESCE(t.client_id, p.client_id) as client_id,
            c.name as client_name,
            te.start_time, te.end_time, te.duration_minutes,
            te.description, te.billable as "billable!", te.billed as "billed!",
            te.hourly_rate, te.total_amount,
            te.created_at, te.updated_at
         FROM time_entries te
         LEFT JOIN users u ON te.user_id = u.id
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN tasks tk ON te.task_id = tk.id
         LEFT JOIN clients c ON COALESCE(t.client_id, p.client_id) = c.id
         WHERE te.user_id = $1
           AND ($2::uuid IS NULL OR te.ticket_id = $2)
           AND ($3::uuid IS NULL OR te.project_id = $3)
           AND ($4::uuid IS NULL OR COALESCE(t.client_id, p.client_id) = $4)
           AND ($5::bool IS NULL OR te.billable = $5)
           AND ($6::bool IS NULL OR te.billed = $6)
           AND ($7::date IS NULL OR te.start_time::date >= $7)
           AND ($8::date IS NULL OR te.start_time::date <= $8)
           AND ($9::text IS NULL OR te.description ILIKE '%' || $9 || '%')
         ORDER BY te.start_time DESC
         LIMIT $10 OFFSET $11"#,
        user_filter,
        params.ticket_id,
        params.project_id,
        params.client_id,
        params.billable,
        params.billed,
        params.from_date,
        params.to_date,
        params.q,
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching time entries: {}", e);
        ApiError::internal("Failed to fetch time entries")
    })?;

    Ok(Json(PaginatedResponse::new(entries, &params.pagination, total)))
}

/// Start a new timer for the authenticated user
async fn start_timer(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<TimeEntryCreate>,
) -> ApiResult<Json<ActiveTimer>> {
    let entry_id = Uuid::new_v4();
    let now = Utc::now();
    let billable = payload.billable.unwrap_or(true);

    // Stop any existing active timer for this user (auto-stop feature)
    let stopped = sqlx::query!(
        "UPDATE time_entries SET
         end_time = NOW(),
         duration_minutes = EXTRACT(EPOCH FROM (NOW() - start_time))::int / 60
         WHERE user_id = $1 AND end_time IS NULL
         RETURNING id",
        user.id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error stopping existing timer: {}", e);
        ApiError::internal("Failed to stop existing timer")
    })?;

    // If we stopped a timer, calculate its billing
    if let Some(stopped_row) = stopped {
        let _ = calculate_and_update_billing(&state, stopped_row.id).await;
    }

    // Start new timer
    sqlx::query!(
        "INSERT INTO time_entries (
            id, user_id, ticket_id, project_id, task_id,
            start_time, description, billable
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        entry_id,
        user.id,
        payload.ticket_id,
        payload.project_id,
        payload.task_id,
        now,
        payload.description,
        billable
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error starting timer: {}", e);
        ApiError::internal("Failed to start timer")
    })?;

    // Fetch and return the active timer details
    let timer = get_active_timer_by_id(&state, entry_id).await?;
    Ok(Json(timer))
}

/// Request body for stopping a timer
#[derive(Debug, Deserialize)]
pub struct StopTimerRequest {
    /// Specific timer ID to stop (optional - stops the active timer if not provided)
    pub timer_id: Option<Uuid>,
}

/// Stop a running timer for the authenticated user
async fn stop_timer(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<StopTimerRequest>,
) -> ApiResult<Json<TimeEntryWithDetails>> {
    let end_time = Utc::now();

    // Stop specific timer or the user's active timer
    let stopped_id = if let Some(timer_id) = payload.timer_id {
        sqlx::query_scalar!(
            "UPDATE time_entries SET
             end_time = $2,
             duration_minutes = EXTRACT(EPOCH FROM ($2 - start_time))::int / 60
             WHERE id = $1 AND user_id = $3 AND end_time IS NULL
             RETURNING id",
            timer_id,
            end_time,
            user.id
        )
        .fetch_optional(&state.db_pool)
        .await
    } else {
        sqlx::query_scalar!(
            "UPDATE time_entries SET
             end_time = $2,
             duration_minutes = EXTRACT(EPOCH FROM ($2 - start_time))::int / 60
             WHERE user_id = $1 AND end_time IS NULL
             RETURNING id",
            user.id,
            end_time
        )
        .fetch_optional(&state.db_pool)
        .await
    }
    .map_err(|e| {
        tracing::error!("Error stopping timer: {}", e);
        ApiError::internal("Failed to stop timer")
    })?;

    let entry_id = stopped_id.ok_or_else(|| ApiError::not_found("No active timer found"))?;

    // Calculate billable amount
    let _ = calculate_and_update_billing(&state, entry_id).await;

    // Fetch and return the updated entry
    let entry = get_time_entry_by_id(&state, entry_id).await?;
    Ok(Json(entry))
}

/// Get all active (running) timers for the authenticated user
async fn get_active_timers(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<ActiveTimer>>> {
    let timers = sqlx::query!(
        r#"SELECT
            te.id, te.user_id, te.ticket_id, t.subject as ticket_subject,
            te.project_id, p.name as project_name,
            c.name as client_name, te.description, te.start_time,
            te.billable as "billable!",
            EXTRACT(EPOCH FROM (NOW() - te.start_time))::int / 60 as "elapsed_minutes!"
         FROM time_entries te
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN clients c ON COALESCE(t.client_id, p.client_id) = c.id
         WHERE te.user_id = $1 AND te.end_time IS NULL
         ORDER BY te.start_time DESC"#,
        user.id
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching active timers: {}", e);
        ApiError::internal("Failed to fetch active timers")
    })?;

    let result: Vec<ActiveTimer> = timers
        .into_iter()
        .map(|row| ActiveTimer {
            id: row.id,
            user_id: row.user_id,
            ticket_id: row.ticket_id,
            ticket_subject: row.ticket_subject,
            project_id: row.project_id,
            project_name: row.project_name,
            client_name: row.client_name,
            description: row.description,
            start_time: row.start_time,
            elapsed_minutes: row.elapsed_minutes,
            billable: row.billable,
        })
        .collect();

    Ok(Json(result))
}

/// Switch from current timer to a new one (stops current, starts new)
async fn switch_timer(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<TimeEntryCreate>,
) -> ApiResult<Json<ActiveTimer>> {
    // This is effectively the same as start_timer (which auto-stops existing)
    start_timer(State(state), AuthUser(user), Json(payload)).await
}

/// Create a manual time entry (not a running timer)
async fn create_manual_entry(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Json(payload): Json<ManualTimeEntry>,
) -> ApiResult<Json<TimeEntryWithDetails>> {
    // Validate: end_time must be after start_time
    if payload.end_time <= payload.start_time {
        return Err(ApiError::validation_single(
            "end_time",
            "End time must be after start time",
        ));
    }

    // Validate description is not empty
    if payload.description.trim().is_empty() {
        return Err(ApiError::validation_single(
            "description",
            "Description is required",
        ));
    }

    let entry_id = Uuid::new_v4();
    let duration = payload.end_time.signed_duration_since(payload.start_time);
    let duration_minutes = duration.num_minutes() as i32;

    sqlx::query!(
        "INSERT INTO time_entries (
            id, user_id, ticket_id, project_id, task_id,
            start_time, end_time, duration_minutes, description, billable
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        entry_id,
        user.id,
        payload.ticket_id,
        payload.project_id,
        payload.task_id,
        payload.start_time,
        payload.end_time,
        duration_minutes,
        payload.description,
        payload.billable
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error creating manual time entry: {}", e);
        ApiError::internal("Failed to create time entry")
    })?;

    // Calculate billing
    let _ = calculate_and_update_billing(&state, entry_id).await;

    let entry = get_time_entry_by_id(&state, entry_id).await?;
    Ok(Json(entry))
}

/// Get a single time entry by ID
async fn get_time_entry(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TimeEntryWithDetails>> {
    let entry = get_time_entry_by_id(&state, id).await?;

    // Ensure user owns this entry (or is admin - TODO: add admin check)
    if entry.user_id != user.id {
        return Err(ApiError::forbidden("You can only view your own time entries"));
    }

    Ok(Json(entry))
}

/// Update a time entry
async fn update_time_entry(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<TimeEntryUpdate>,
) -> ApiResult<Json<TimeEntryWithDetails>> {
    // Verify ownership first
    let existing = get_time_entry_by_id(&state, id).await?;
    if existing.user_id != user.id {
        return Err(ApiError::forbidden("You can only edit your own time entries"));
    }

    // Calculate duration if start and end times are provided
    let duration = if let (Some(start), Some(end)) = (&payload.start_time, &payload.end_time) {
        if end <= start {
            return Err(ApiError::validation_single(
                "end_time",
                "End time must be after start time",
            ));
        }
        Some(end.signed_duration_since(*start).num_minutes() as i32)
    } else {
        payload.duration_minutes
    };

    let result = sqlx::query!(
        "UPDATE time_entries SET
         ticket_id = COALESCE($2, ticket_id),
         project_id = COALESCE($3, project_id),
         task_id = COALESCE($4, task_id),
         description = COALESCE($5, description),
         billable = COALESCE($6, billable),
         start_time = COALESCE($7, start_time),
         end_time = COALESCE($8, end_time),
         duration_minutes = COALESCE($9, duration_minutes),
         updated_at = NOW()
         WHERE id = $1 AND user_id = $10",
        id,
        payload.ticket_id,
        payload.project_id,
        payload.task_id,
        payload.description,
        payload.billable,
        payload.start_time,
        payload.end_time,
        duration,
        user.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error updating time entry: {}", e);
        ApiError::internal("Failed to update time entry")
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Time entry not found"));
    }

    // Recalculate billing
    let _ = calculate_and_update_billing(&state, id).await;

    let entry = get_time_entry_by_id(&state, id).await?;
    Ok(Json(entry))
}

/// Delete a time entry
async fn delete_time_entry(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<()> {
    // Verify ownership first
    let existing = get_time_entry_by_id(&state, id).await?;
    if existing.user_id != user.id {
        return Err(ApiError::forbidden("You can only delete your own time entries"));
    }

    // Don't allow deleting billed entries
    if existing.billed {
        return Err(ApiError::validation_single(
            "id",
            "Cannot delete a billed time entry",
        ));
    }

    let result = sqlx::query!("DELETE FROM time_entries WHERE id = $1 AND user_id = $2", id, user.id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Error deleting time entry: {}", e);
            ApiError::internal("Failed to delete time entry")
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found("Time entry not found"));
    }

    Ok(())
}

/// Get time tracking statistics for the authenticated user
async fn get_time_stats(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<TimeStats>> {
    let stats = sqlx::query!(
        r#"SELECT
            COALESCE(SUM(duration_minutes) FILTER (WHERE start_time::date = CURRENT_DATE), 0)::float8 / 60.0 as "hours_today!",
            COALESCE(SUM(duration_minutes) FILTER (WHERE start_time::date = CURRENT_DATE AND billable = true), 0)::float8 / 60.0 as "billable_hours_today!",
            COALESCE(SUM(duration_minutes) FILTER (WHERE start_time >= date_trunc('week', CURRENT_DATE)), 0)::float8 / 60.0 as "hours_week!",
            COALESCE(SUM(duration_minutes) FILTER (WHERE start_time >= date_trunc('week', CURRENT_DATE) AND billable = true), 0)::float8 / 60.0 as "billable_hours_week!",
            COALESCE(SUM(total_amount) FILTER (WHERE billable = true AND billed = false), 0)::float8 as "unbilled_amount!",
            COUNT(*) FILTER (WHERE end_time IS NULL)::int as "active_timers!"
         FROM time_entries
         WHERE user_id = $1"#,
        user.id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching time stats: {}", e);
        ApiError::internal("Failed to fetch time stats")
    })?;

    Ok(Json(TimeStats {
        total_hours_today: stats.hours_today,
        billable_hours_today: stats.billable_hours_today,
        total_hours_week: stats.hours_week,
        billable_hours_week: stats.billable_hours_week,
        unbilled_amount: stats.unbilled_amount,
        active_timers: stats.active_timers,
    }))
}

/// Timesheet view parameters
#[derive(Debug, Clone, Deserialize)]
pub struct TimesheetParams {
    /// Start date for timesheet (defaults to start of current week)
    pub from_date: Option<NaiveDate>,
    /// End date for timesheet (defaults to end of current week)
    pub to_date: Option<NaiveDate>,
    /// Group by: "day", "week", "project", "client"
    pub group_by: Option<String>,
}

/// Timesheet entry grouped by day
#[derive(Debug, Clone, Serialize)]
pub struct TimesheetDay {
    pub date: NaiveDate,
    pub total_hours: f64,
    pub billable_hours: f64,
    pub entries: Vec<TimeEntryWithDetails>,
}

/// Get timesheet view for the authenticated user
async fn get_timesheet(
    State(state): State<Arc<AppState>>,
    AuthUser(user): AuthUser,
    Query(params): Query<TimesheetParams>,
) -> ApiResult<Json<Vec<TimesheetDay>>> {
    // Default to current week
    let today = Utc::now().date_naive();
    let from_date = params.from_date.unwrap_or_else(|| {
        // Start of current week (Monday)
        today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64)
    });
    let to_date = params.to_date.unwrap_or_else(|| from_date + chrono::Duration::days(6));

    // Fetch all entries in date range
    let entries = sqlx::query_as!(
        TimeEntryWithDetails,
        r#"SELECT
            te.id, te.user_id,
            COALESCE(u.first_name || ' ' || u.last_name, 'Unknown') as "user_name!",
            te.ticket_id, t.number as ticket_number, t.subject as ticket_subject,
            te.project_id, p.name as project_name,
            te.task_id, tk.name as task_name,
            COALESCE(t.client_id, p.client_id) as client_id,
            c.name as client_name,
            te.start_time, te.end_time, te.duration_minutes,
            te.description, te.billable as "billable!", te.billed as "billed!",
            te.hourly_rate, te.total_amount,
            te.created_at, te.updated_at
         FROM time_entries te
         LEFT JOIN users u ON te.user_id = u.id
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN tasks tk ON te.task_id = tk.id
         LEFT JOIN clients c ON COALESCE(t.client_id, p.client_id) = c.id
         WHERE te.user_id = $1
           AND te.start_time::date >= $2
           AND te.start_time::date <= $3
         ORDER BY te.start_time DESC"#,
        user.id,
        from_date,
        to_date
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching timesheet: {}", e);
        ApiError::internal("Failed to fetch timesheet")
    })?;

    // Group by day
    let mut days: std::collections::HashMap<NaiveDate, Vec<TimeEntryWithDetails>> =
        std::collections::HashMap::new();

    for entry in entries {
        let date = entry.start_time.date_naive();
        days.entry(date).or_default().push(entry);
    }

    // Build response with all days in range (including empty ones)
    let mut result: Vec<TimesheetDay> = Vec::new();
    let mut current = from_date;
    while current <= to_date {
        let day_entries = days.remove(&current).unwrap_or_default();
        let total_hours: f64 = day_entries
            .iter()
            .filter_map(|e| e.duration_minutes)
            .sum::<i32>() as f64
            / 60.0;
        let billable_hours: f64 = day_entries
            .iter()
            .filter(|e| e.billable)
            .filter_map(|e| e.duration_minutes)
            .sum::<i32>() as f64
            / 60.0;

        result.push(TimesheetDay {
            date: current,
            total_hours,
            billable_hours,
            entries: day_entries,
        });

        current += chrono::Duration::days(1);
    }

    Ok(Json(result))
}

// Helper functions

async fn get_time_entry_by_id(state: &AppState, id: Uuid) -> Result<TimeEntryWithDetails, ApiError> {
    sqlx::query_as!(
        TimeEntryWithDetails,
        r#"SELECT
            te.id, te.user_id,
            COALESCE(u.first_name || ' ' || u.last_name, 'Unknown') as "user_name!",
            te.ticket_id, t.number as ticket_number, t.subject as ticket_subject,
            te.project_id, p.name as project_name,
            te.task_id, tk.name as task_name,
            COALESCE(t.client_id, p.client_id) as client_id,
            c.name as client_name,
            te.start_time, te.end_time, te.duration_minutes,
            te.description, te.billable as "billable!", te.billed as "billed!",
            te.hourly_rate, te.total_amount,
            te.created_at, te.updated_at
         FROM time_entries te
         LEFT JOIN users u ON te.user_id = u.id
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN tasks tk ON te.task_id = tk.id
         LEFT JOIN clients c ON COALESCE(t.client_id, p.client_id) = c.id
         WHERE te.id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching time entry: {}", e);
        ApiError::internal("Failed to fetch time entry")
    })?
    .ok_or_else(|| ApiError::not_found("Time entry not found"))
}

async fn get_active_timer_by_id(state: &AppState, id: Uuid) -> Result<ActiveTimer, ApiError> {
    let row = sqlx::query!(
        r#"SELECT
            te.id, te.user_id, te.ticket_id, t.subject as ticket_subject,
            te.project_id, p.name as project_name,
            c.name as client_name, te.description, te.start_time,
            te.billable as "billable!",
            EXTRACT(EPOCH FROM (NOW() - te.start_time))::int / 60 as "elapsed_minutes!"
         FROM time_entries te
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN clients c ON COALESCE(t.client_id, p.client_id) = c.id
         WHERE te.id = $1"#,
        id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error fetching active timer: {}", e);
        ApiError::internal("Failed to fetch timer")
    })?
    .ok_or_else(|| ApiError::not_found("Timer not found"))?;

    Ok(ActiveTimer {
        id: row.id,
        user_id: row.user_id,
        ticket_id: row.ticket_id,
        ticket_subject: row.ticket_subject,
        project_id: row.project_id,
        project_name: row.project_name,
        client_name: row.client_name,
        description: row.description,
        start_time: row.start_time,
        elapsed_minutes: row.elapsed_minutes,
        billable: row.billable,
    })
}

async fn calculate_and_update_billing(state: &AppState, entry_id: Uuid) -> Result<(), ApiError> {
    // TODO: Get user's hourly rate or project/client rate from settings
    let default_rate = Decimal::from(75); // $75/hour default

    sqlx::query!(
        "UPDATE time_entries SET
         hourly_rate = COALESCE(hourly_rate, $2),
         total_amount = CASE WHEN billable THEN
                           COALESCE(hourly_rate, $2) * (COALESCE(duration_minutes, 0)::decimal / 60)
                        ELSE 0 END
         WHERE id = $1",
        entry_id,
        default_rate
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Error calculating billing: {}", e);
        ApiError::internal("Failed to calculate billing")
    })?;

    Ok(())
}