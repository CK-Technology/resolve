use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;
use crate::AppState;

#[derive(Serialize, Deserialize)]
pub struct ProjectCreate {
    pub client_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub budget: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub project_manager_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub budget: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub project_manager_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskCreate {
    pub project_id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub priority: Option<String>,
    pub estimated_hours: Option<Decimal>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub estimated_hours: Option<Decimal>,
    pub actual_hours: Option<Decimal>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub client_id: Option<Uuid>,
    pub status: Option<String>,
    pub project_manager_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectWithDetails {
    pub id: Uuid,
    pub client_id: Uuid,
    pub client_name: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub budget: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub project_manager_id: Option<Uuid>,
    pub project_manager_name: Option<String>,
    pub total_hours: Option<Decimal>,
    pub billable_hours: Option<Decimal>,
    pub total_cost: Option<Decimal>,
    pub task_count: i64,
    pub completed_tasks: i64,
    pub open_tickets: i64,
    pub progress_percentage: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskWithDetails {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_name: String,
    pub ticket_id: Option<Uuid>,
    pub ticket_number: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_name: Option<String>,
    pub status: String,
    pub priority: String,
    pub estimated_hours: Option<Decimal>,
    pub actual_hours: Option<Decimal>,
    pub time_logged: Option<Decimal>,
    pub due_date: Option<NaiveDate>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

pub fn project_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/:id", get(get_project).put(update_project).delete(delete_project))
        .route("/:id/tasks", get(get_project_tasks).post(create_task))
        .route("/:id/time-entries", get(get_project_time_entries))
        .route("/:id/stats", get(get_project_stats))
        .route("/tasks/:task_id", get(get_task).put(update_task).delete(delete_task))
}

async fn list_projects(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProjectQuery>,
) -> Result<Json<Vec<ProjectWithDetails>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    // Build dynamic query based on filters
    let mut where_clauses = vec!["1=1"];
    let mut param_values = vec![];
    let mut param_count = 1;
    
    if let Some(client_id) = params.client_id {
        where_clauses.push(&format!("p.client_id = ${}", param_count));
        param_values.push(client_id.to_string());
        param_count += 1;
    }
    
    if let Some(status) = &params.status {
        where_clauses.push(&format!("p.status = ${}", param_count));
        param_values.push(status.clone());
        param_count += 1;
    }
    
    if let Some(manager_id) = params.project_manager_id {
        where_clauses.push(&format!("p.project_manager_id = ${}", param_count));
        param_values.push(manager_id.to_string());
        param_count += 1;
    }
    
    // For now, use a simplified query without dynamic parameters
    match sqlx::query_as!(
        ProjectWithDetails,
        "SELECT 
            p.id, p.client_id, c.name as client_name,
            p.name, p.description, p.status,
            p.start_date, p.end_date, p.budget, p.hourly_rate,
            p.project_manager_id,
            CASE WHEN u.id IS NOT NULL THEN u.first_name || ' ' || u.last_name ELSE NULL END as project_manager_name,
            COALESCE(te_stats.total_hours, 0) as total_hours,
            COALESCE(te_stats.billable_hours, 0) as billable_hours,
            COALESCE(te_stats.total_cost, 0) as total_cost,
            COALESCE(task_stats.task_count, 0) as task_count,
            COALESCE(task_stats.completed_tasks, 0) as completed_tasks,
            COALESCE(ticket_stats.open_tickets, 0) as open_tickets,
            CASE WHEN COALESCE(task_stats.task_count, 0) > 0 
                 THEN (COALESCE(task_stats.completed_tasks, 0) * 100 / task_stats.task_count)::int
                 ELSE 0 END as progress_percentage,
            p.created_at, p.updated_at
         FROM projects p
         LEFT JOIN clients c ON p.client_id = c.id
         LEFT JOIN users u ON p.project_manager_id = u.id
         LEFT JOIN (
            SELECT 
                project_id,
                SUM(duration_minutes) / 60.0 as total_hours,
                SUM(CASE WHEN billable THEN duration_minutes ELSE 0 END) / 60.0 as billable_hours,
                SUM(total_amount) as total_cost
            FROM time_entries 
            WHERE project_id IS NOT NULL
            GROUP BY project_id
         ) te_stats ON p.id = te_stats.project_id
         LEFT JOIN (
            SELECT 
                project_id,
                COUNT(*) as task_count,
                COUNT(*) FILTER (WHERE status = 'completed') as completed_tasks
            FROM tasks 
            GROUP BY project_id
         ) task_stats ON p.id = task_stats.project_id
         LEFT JOIN (
            SELECT 
                p.id as project_id,
                COUNT(t.*) as open_tickets
            FROM projects p
            LEFT JOIN tasks task ON p.id = task.project_id
            LEFT JOIN tickets t ON task.ticket_id = t.id AND t.status NOT IN ('closed', 'resolved')
            GROUP BY p.id
         ) ticket_stats ON p.id = ticket_stats.project_id
         ORDER BY p.created_at DESC
         LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(projects) => Ok(Json(projects)),
        Err(e) => {
            tracing::error!("Error fetching projects: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_project(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProjectCreate>,
) -> Result<(StatusCode, Json<ProjectWithDetails>), StatusCode> {
    let project_id = Uuid::new_v4();
    
    match sqlx::query!(
        "INSERT INTO projects (
            id, client_id, name, description, start_date, end_date,
            budget, hourly_rate, project_manager_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        project_id,
        payload.client_id,
        payload.name,
        payload.description,
        payload.start_date,
        payload.end_date,
        payload.budget,
        payload.hourly_rate,
        payload.project_manager_id
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(_) => {
            match get_project_by_id(&state, project_id).await {
                Ok(project) => Ok((StatusCode::CREATED, Json(project))),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(e) => {
            tracing::error!("Error creating project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectWithDetails>, StatusCode> {
    match get_project_by_id(&state, id).await {
        Ok(project) => Ok(Json(project)),
        Err(StatusCode::NOT_FOUND) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ProjectUpdate>,
) -> Result<Json<ProjectWithDetails>, StatusCode> {
    match sqlx::query!(
        "UPDATE projects SET 
         name = COALESCE($2, name),
         description = COALESCE($3, description),
         status = COALESCE($4, status),
         start_date = COALESCE($5, start_date),
         end_date = COALESCE($6, end_date),
         budget = COALESCE($7, budget),
         hourly_rate = COALESCE($8, hourly_rate),
         project_manager_id = COALESCE($9, project_manager_id),
         updated_at = NOW()
         WHERE id = $1",
        id,
        payload.name,
        payload.description,
        payload.status,
        payload.start_date,
        payload.end_date,
        payload.budget,
        payload.hourly_rate,
        payload.project_manager_id
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                match get_project_by_id(&state, id).await {
                    Ok(project) => Ok(Json(project)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error updating project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_project(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match sqlx::query!("DELETE FROM projects WHERE id = $1", id)
        .execute(&state.db_pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error deleting project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project_tasks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TaskWithDetails>>, StatusCode> {
    match sqlx::query_as!(
        TaskWithDetails,
        "SELECT 
            t.id, t.project_id, p.name as project_name,
            t.ticket_id, tk.number as ticket_number,
            t.name, t.description,
            t.assigned_to,
            CASE WHEN u.id IS NOT NULL THEN u.first_name || ' ' || u.last_name ELSE NULL END as assigned_name,
            t.status, t.priority,
            t.estimated_hours, t.actual_hours,
            COALESCE(te_stats.time_logged, 0) as time_logged,
            t.due_date, t.completed_at,
            t.created_at, t.updated_at
         FROM tasks t
         LEFT JOIN projects p ON t.project_id = p.id
         LEFT JOIN tickets tk ON t.ticket_id = tk.id
         LEFT JOIN users u ON t.assigned_to = u.id
         LEFT JOIN (
            SELECT 
                task_id,
                SUM(duration_minutes) / 60.0 as time_logged
            FROM time_entries 
            WHERE task_id IS NOT NULL
            GROUP BY task_id
         ) te_stats ON t.id = te_stats.task_id
         WHERE t.project_id = $1
         ORDER BY t.created_at DESC",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(tasks) => Ok(Json(tasks)),
        Err(e) => {
            tracing::error!("Error fetching project tasks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TaskCreate>,
) -> Result<(StatusCode, Json<TaskWithDetails>), StatusCode> {
    let task_id = Uuid::new_v4();
    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());
    
    match sqlx::query!(
        "INSERT INTO tasks (
            id, project_id, ticket_id, name, description,
            assigned_to, priority, estimated_hours, due_date
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        task_id,
        payload.project_id,
        payload.ticket_id,
        payload.name,
        payload.description,
        payload.assigned_to,
        priority,
        payload.estimated_hours,
        payload.due_date
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(_) => {
            match get_task_by_id(&state, task_id).await {
                Ok(task) => Ok((StatusCode::CREATED, Json(task))),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(e) => {
            tracing::error!("Error creating task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskWithDetails>, StatusCode> {
    match get_task_by_id(&state, task_id).await {
        Ok(task) => Ok(Json(task)),
        Err(StatusCode::NOT_FOUND) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<TaskUpdate>,
) -> Result<Json<TaskWithDetails>, StatusCode> {
    // If status is being changed to completed, set completed_at
    let completed_at = if payload.status.as_deref() == Some("completed") {
        Some(Utc::now())
    } else {
        None
    };
    
    match sqlx::query!(
        "UPDATE tasks SET 
         name = COALESCE($2, name),
         description = COALESCE($3, description),
         assigned_to = COALESCE($4, assigned_to),
         status = COALESCE($5, status),
         priority = COALESCE($6, priority),
         estimated_hours = COALESCE($7, estimated_hours),
         actual_hours = COALESCE($8, actual_hours),
         due_date = COALESCE($9, due_date),
         completed_at = COALESCE($10, completed_at),
         updated_at = NOW()
         WHERE id = $1",
        task_id,
        payload.name,
        payload.description,
        payload.assigned_to,
        payload.status,
        payload.priority,
        payload.estimated_hours,
        payload.actual_hours,
        payload.due_date,
        completed_at
    )
    .execute(&state.db_pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                match get_task_by_id(&state, task_id).await {
                    Ok(task) => Ok(Json(task)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error updating task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    match sqlx::query!("DELETE FROM tasks WHERE id = $1", task_id)
        .execute(&state.db_pool)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Error deleting task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project_time_entries(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<crate::handlers::time_tracking::TimeEntryWithDetails>>, StatusCode> {
    // Reuse the time tracking handler logic
    match sqlx::query_as!(
        crate::handlers::time_tracking::TimeEntryWithDetails,
        "SELECT 
            te.id, te.user_id, u.first_name || ' ' || u.last_name as user_name,
            te.ticket_id, t.number as ticket_number, t.subject as ticket_subject,
            te.project_id, p.name as project_name,
            te.task_id, tk.name as task_name,
            COALESCE(t.client_id, p.client_id) as client_id,
            c.name as client_name,
            te.start_time, te.end_time, te.duration_minutes,
            te.description, te.billable, te.billed,
            te.hourly_rate, te.total_amount,
            te.created_at, te.updated_at
         FROM time_entries te
         LEFT JOIN users u ON te.user_id = u.id
         LEFT JOIN tickets t ON te.ticket_id = t.id
         LEFT JOIN projects p ON te.project_id = p.id
         LEFT JOIN tasks tk ON te.task_id = tk.id
         LEFT JOIN clients c ON COALESCE(t.client_id, p.client_id) = c.id
         WHERE te.project_id = $1
         ORDER BY te.start_time DESC",
        id
    )
    .fetch_all(&state.db_pool)
    .await
    {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => {
            tracing::error!("Error fetching project time entries: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
pub struct ProjectStats {
    pub total_hours: Decimal,
    pub billable_hours: Decimal,
    pub total_cost: Decimal,
    pub budget_remaining: Option<Decimal>,
    pub budget_utilized_percent: Option<i32>,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub overdue_tasks: i64,
    pub team_members: i64,
}

async fn get_project_stats(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectStats>, StatusCode> {
    // Get project budget for calculations
    let project = match sqlx::query!("SELECT budget FROM projects WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await
    {
        Ok(Some(project)) => project,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Error fetching project: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let stats = match sqlx::query!(
        "SELECT 
            COALESCE(SUM(te.duration_minutes) / 60.0, 0) as total_hours,
            COALESCE(SUM(CASE WHEN te.billable THEN te.duration_minutes ELSE 0 END) / 60.0, 0) as billable_hours,
            COALESCE(SUM(te.total_amount), 0) as total_cost,
            COUNT(DISTINCT t.id) as total_tasks,
            COUNT(DISTINCT t.id) FILTER (WHERE t.status = 'completed') as completed_tasks,
            COUNT(DISTINCT t.id) FILTER (WHERE t.due_date < CURRENT_DATE AND t.status != 'completed') as overdue_tasks,
            COUNT(DISTINCT te.user_id) as team_members
         FROM projects p
         LEFT JOIN time_entries te ON p.id = te.project_id
         LEFT JOIN tasks t ON p.id = t.project_id
         WHERE p.id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(row) => {
            let total_cost = row.total_cost.unwrap_or_default();
            let budget_remaining = project.budget.map(|budget| budget - total_cost);
            let budget_utilized_percent = project.budget.map(|budget| {
                if budget > Decimal::ZERO {
                    ((total_cost / budget) * Decimal::from(100)).to_i32().unwrap_or(0).min(100)
                } else {
                    0
                }
            });
            
            ProjectStats {
                total_hours: Decimal::from_f64_retain(row.total_hours.unwrap_or(0.0)).unwrap_or_default(),
                billable_hours: Decimal::from_f64_retain(row.billable_hours.unwrap_or(0.0)).unwrap_or_default(),
                total_cost,
                budget_remaining,
                budget_utilized_percent,
                total_tasks: row.total_tasks.unwrap_or(0),
                completed_tasks: row.completed_tasks.unwrap_or(0),
                overdue_tasks: row.overdue_tasks.unwrap_or(0),
                team_members: row.team_members.unwrap_or(0),
            }
        }
        Err(e) => {
            tracing::error!("Error fetching project stats: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    Ok(Json(stats))
}

// Helper functions

async fn get_project_by_id(state: &AppState, id: Uuid) -> Result<ProjectWithDetails, StatusCode> {
    match sqlx::query_as!(
        ProjectWithDetails,
        "SELECT 
            p.id, p.client_id, c.name as client_name,
            p.name, p.description, p.status,
            p.start_date, p.end_date, p.budget, p.hourly_rate,
            p.project_manager_id,
            CASE WHEN u.id IS NOT NULL THEN u.first_name || ' ' || u.last_name ELSE NULL END as project_manager_name,
            COALESCE(te_stats.total_hours, 0) as total_hours,
            COALESCE(te_stats.billable_hours, 0) as billable_hours,
            COALESCE(te_stats.total_cost, 0) as total_cost,
            COALESCE(task_stats.task_count, 0) as task_count,
            COALESCE(task_stats.completed_tasks, 0) as completed_tasks,
            0 as open_tickets, -- TODO: Implement proper ticket counting
            CASE WHEN COALESCE(task_stats.task_count, 0) > 0 
                 THEN (COALESCE(task_stats.completed_tasks, 0) * 100 / task_stats.task_count)::int
                 ELSE 0 END as progress_percentage,
            p.created_at, p.updated_at
         FROM projects p
         LEFT JOIN clients c ON p.client_id = c.id
         LEFT JOIN users u ON p.project_manager_id = u.id
         LEFT JOIN (
            SELECT 
                project_id,
                SUM(duration_minutes) / 60.0 as total_hours,
                SUM(CASE WHEN billable THEN duration_minutes ELSE 0 END) / 60.0 as billable_hours,
                SUM(total_amount) as total_cost
            FROM time_entries 
            WHERE project_id = $1
            GROUP BY project_id
         ) te_stats ON p.id = te_stats.project_id
         LEFT JOIN (
            SELECT 
                project_id,
                COUNT(*) as task_count,
                COUNT(*) FILTER (WHERE status = 'completed') as completed_tasks
            FROM tasks 
            WHERE project_id = $1
            GROUP BY project_id
         ) task_stats ON p.id = task_stats.project_id
         WHERE p.id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(project) => Ok(project),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Error fetching project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_task_by_id(state: &AppState, id: Uuid) -> Result<TaskWithDetails, StatusCode> {
    match sqlx::query_as!(
        TaskWithDetails,
        "SELECT 
            t.id, t.project_id, p.name as project_name,
            t.ticket_id, tk.number as ticket_number,
            t.name, t.description,
            t.assigned_to,
            CASE WHEN u.id IS NOT NULL THEN u.first_name || ' ' || u.last_name ELSE NULL END as assigned_name,
            t.status, t.priority,
            t.estimated_hours, t.actual_hours,
            COALESCE(te_stats.time_logged, 0) as time_logged,
            t.due_date, t.completed_at,
            t.created_at, t.updated_at
         FROM tasks t
         LEFT JOIN projects p ON t.project_id = p.id
         LEFT JOIN tickets tk ON t.ticket_id = tk.id
         LEFT JOIN users u ON t.assigned_to = u.id
         LEFT JOIN (
            SELECT 
                task_id,
                SUM(duration_minutes) / 60.0 as time_logged
            FROM time_entries 
            WHERE task_id = $1
            GROUP BY task_id
         ) te_stats ON t.id = te_stats.task_id
         WHERE t.id = $1",
        id
    )
    .fetch_one(&state.db_pool)
    .await
    {
        Ok(task) => Ok(task),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Error fetching task: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}