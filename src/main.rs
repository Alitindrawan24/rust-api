use axum::{
  extract::{Path, State},
  http::StatusCode,
  routing::{get},
  Json, Router,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
  // expose the environment variables
  dotenvy::dotenv().expect("Unable to access .env file");

  // set variables from the environment variables
  let server_address: String =
    std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_owned());
  let database_url: String =
    std::env::var("DATABASE_URL").expect("DATABASE_URL not found in environment variable");

  // create the database pool
  let db_pool = PgPoolOptions::new()
    .max_connections(16)
    .connect(&database_url)
    .await
    .expect("Can't create database pool");

  // create our TCP listener
  let listener = TcpListener::bind(server_address)
    .await
    .expect("Could not create TCP listener");

  println!("listening on {}", listener.local_addr().unwrap());

  // compose the routes
  let app = Router::new()
    .route("/", get(|| async { "Hello World" })) 
    .route("/api/tasks", get(get_tasks).post(create_task))
    .route("/api/tasks/:id", get(get_detail_task).patch(update_task).delete(delete_task))
    .with_state(db_pool);

  // serve the application
  axum::serve(listener, app)
    .await
    .expect("Failed to serve the application");
}

#[derive(Serialize)]
struct TaskRow {
  id: i32,
  name: String,
  priority: Option<i32>,
}

async fn get_tasks(
  State(pg_pool): State<PgPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
  let rows = sqlx::query_as!(TaskRow, "SELECT * FROM tasks ORDER BY id DESC")
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string() }).to_string(),
      )
    })?;

  Ok((
    StatusCode::OK,
    json!({ "success": true, "data": rows }).to_string(),
  ))
}

async fn get_detail_task(
  State(pg_pool): State<PgPool>,
  Path(id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
  let row = sqlx::query_as!(TaskRow, "SELECT * FROM tasks WHERE id = $1", id)
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string() }).to_string(),
      )
    })?;

  Ok((
    StatusCode::OK,
    json!({ "success": true, "data": row }).to_string(),
  ))
}

#[derive(Deserialize)]
struct CreateTaskRequest {
  name: String,
  priority: Option<i32>,
}

#[derive(Serialize)]
struct CreateTaskRow {
  id: i32,
}

async fn create_task(
  State(pg_pool): State<PgPool>,
  Json(task): Json<CreateTaskRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
  let row = sqlx::query_as!(
    CreateTaskRow,
    "INSERT INTO tasks (name, priority) VALUES ($1, $2) RETURNING id",
    task.name,
    task.priority
  )
  .fetch_one(&pg_pool)
  .await
  .map_err(|e| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      json!({ "success": false, "message": e.to_string() }).to_string(),
    )
  })?;

  Ok((
    StatusCode::CREATED,
    json!({ "success": true, "data": row }).to_string(),
  ))
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
  name: String,
  priority: Option<i32>,
}

async fn update_task(
  State(pg_pool): State<PgPool>,
  Path(id): Path<i32>,
  Json(task): Json<UpdateTaskRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
  sqlx::query!(
    "UPDATE tasks SET name = $2, priority = $3 WHERE id = $1",
    id,
    task.name,
    task.priority
  )
  .execute(&pg_pool)
  .await
  .map_err(|e| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      json!({ "success": false, "message": e.to_string() }).to_string(),
    )
  })?;

  Ok((StatusCode::OK, json!({ "success": true }).to_string()))
}

async fn delete_task(
  State(pg_pool): State<PgPool>,
  Path(id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
  sqlx::query!("DELETE FROM tasks WHERE id = $1", id)
    .execute(&pg_pool)
    .await
    .map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string() }).to_string(),
      )
    })?;

  Ok((StatusCode::OK, json!({ "success": true }).to_string()))
}
