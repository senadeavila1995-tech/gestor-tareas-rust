use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Serialize)]
struct ApiResponse {
    status: &'static str,
    application: &'static str,
    language: &'static str,
}

#[derive(Deserialize)]
struct CreateTask {
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    due_date: Option<NaiveDate>,
}

#[derive(Deserialize)]
struct UpdateTask {
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
    due_date: Option<NaiveDate>,
}

#[derive(Serialize, sqlx::FromRow)]
struct Task {
    id: i64,
    title: String,
    description: Option<String>,
    status: String,
    priority: String,
    due_date: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub(crate) fn validate_task(
    title: &str,
    status: &str,
    priority: &str,
) -> Result<(), (StatusCode, String)> {
    if title.trim().is_empty() || title.chars().count() > 150 {
        return Err((
            StatusCode::BAD_REQUEST,
            "El título es obligatorio y debe tener máximo 150 caracteres".into(),
        ));
    }

    if !["pending", "in_progress", "completed"].contains(&status) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Estado inválido: pending, in_progress o completed".into(),
        ));
    }

    if !["low", "medium", "high", "urgent"].contains(&priority) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Prioridad inválida: low, medium, high o urgent".into(),
        ));
    }

    Ok(())
}

async fn root() -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "online",
        application: "Gestor de Tareas",
        language: "Rust + Axum",
    })
}

async fn health() -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "ok",
        application: "Gestor de Tareas",
        language: "Rust + PostgreSQL",
    })
}

async fn list_tasks(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, title, description, status, priority,
               due_date, created_at, updated_at
        FROM tasks
        ORDER BY created_at DESC, id DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| {
        eprintln!("Error consultando tareas: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(tasks))
}

async fn create_task(
    State(pool): State<PgPool>,
    Json(input): Json<CreateTask>,
) -> Result<(StatusCode, Json<Task>), (StatusCode, String)> {
    let title = input.title.trim();
    let status = input.status.unwrap_or_else(|| "pending".into());
    let priority = input.priority.unwrap_or_else(|| "medium".into());

    validate_task(title, &status, &priority)?;

    let task = sqlx::query_as::<_, Task>(
        r#"
        INSERT INTO tasks (title, description, status, priority, due_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, title, description, status, priority,
                  due_date, created_at, updated_at
        "#,
    )
    .bind(title)
    .bind(input.description)
    .bind(status)
    .bind(priority)
    .bind(input.due_date)
    .fetch_one(&pool)
    .await
    .map_err(|error| {
        eprintln!("Error creando tarea: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "No se pudo crear la tarea".into(),
        )
    })?;

    Ok((StatusCode::CREATED, Json(task)))
}

async fn update_task(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<UpdateTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    let title = input.title.trim();
    let status = input.status.unwrap_or_else(|| "pending".into());
    let priority = input.priority.unwrap_or_else(|| "medium".into());

    validate_task(title, &status, &priority)?;

    let task = sqlx::query_as::<_, Task>(
        r#"
        UPDATE tasks
        SET title = $1,
            description = $2,
            status = $3,
            priority = $4,
            due_date = $5,
            updated_at = NOW()
        WHERE id = $6
        RETURNING id, title, description, status, priority,
                  due_date, created_at, updated_at
        "#,
    )
    .bind(title)
    .bind(input.description)
    .bind(status)
    .bind(priority)
    .bind(input.due_date)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|error| {
        eprintln!("Error actualizando tarea: {error}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "No se pudo actualizar la tarea".into(),
        )
    })?;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err((StatusCode::NOT_FOUND, "Tarea no encontrada".into())),
    }
}

async fn delete_task(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|error| {
            eprintln!("Error eliminando tarea: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No se pudo eliminar la tarea".into(),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Tarea no encontrada".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("Falta configurar DATABASE_URL en .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("No se pudo conectar con PostgreSQL");

    println!("Conexión con PostgreSQL establecida.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("No se pudieron ejecutar las migraciones");

    println!("Migraciones ejecutadas correctamente.");

    let app = Router::new()
        .route("/", get(root))
        .route("/api/health", get(health))
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route("/api/tasks/{id}", put(update_task).delete(delete_task))
        .with_state(pool)
        .layer(CorsLayer::permissive());

    let address = SocketAddr::from(([127, 0, 0, 1], 8101));

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("No se pudo abrir el puerto 8101");

    println!("API Rust: http://127.0.0.1:8101");
    println!("Salud: http://127.0.0.1:8101/api/health");
    println!("Tareas: http://127.0.0.1:8101/api/tasks");

    axum::serve(listener, app)
        .await
        .expect("Error ejecutando la API");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acepta_tarea_valida() {
        assert!(validate_task("Aprender Rust", "pending", "medium").is_ok());
    }

    #[test]
    fn rechaza_titulo_vacio() {
        assert!(validate_task("   ", "pending", "medium").is_err());
    }

    #[test]
    fn rechaza_titulo_mayor_a_150_caracteres() {
        let title = "a".repeat(151);
        assert!(validate_task(&title, "pending", "medium").is_err());
    }

    #[test]
    fn acepta_titulo_de_150_caracteres() {
        let title = "a".repeat(150);
        assert!(validate_task(&title, "pending", "medium").is_ok());
    }

    #[test]
    fn rechaza_estado_invalido() {
        assert!(validate_task("Tarea", "invalido", "medium").is_err());
    }

    #[test]
    fn acepta_estados_permitidos() {
        for status in ["pending", "in_progress", "completed"] {
            assert!(validate_task("Tarea", status, "medium").is_ok());
        }
    }

    #[test]
    fn rechaza_prioridad_invalida() {
        assert!(validate_task("Tarea", "pending", "critica").is_err());
    }

    #[test]
    fn acepta_prioridades_permitidas() {
        for priority in ["low", "medium", "high", "urgent"] {
            assert!(validate_task("Tarea", "pending", priority).is_ok());
        }
    }
}
