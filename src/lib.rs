use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;

#[derive(Clone)]
struct AppState(Arc<Mutex<Store>>);

struct Store {
    tasks: BTreeMap<String, Task>,
    next_id: u64,
}

#[derive(Clone, Serialize)]
struct Task {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Deserialize)]
struct CreateTask {
    title: String,
}

#[derive(Deserialize)]
struct UpdateTask {
    title: Option<String>,
    completed: Option<bool>,
}

pub fn app() -> Router {
    let state = AppState(Arc::new(Mutex::new(Store {
        tasks: BTreeMap::new(),
        next_id: 1,
    })));
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).patch(update_task).delete(delete_task),
        )
        .layer(DefaultBodyLimit::max(1_000_000))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let store = state.0.lock().expect("task store poisoned");
    Json(store.tasks.values().cloned().collect())
}

async fn create_task(
    State(state): State<AppState>,
    payload: Result<Json<CreateTask>, JsonRejection>,
) -> Result<(StatusCode, Json<Task>), ApiError> {
    let body = json_body(payload)?;
    let title = body.title.trim();
    if title.is_empty() {
        return Err(ApiError::bad_request("title is required"));
    }

    let mut store = state.0.lock().expect("task store poisoned");
    let task = Task {
        id: store.next_id.to_string(),
        title: title.to_owned(),
        completed: false,
    };
    store.next_id += 1;
    store.tasks.insert(task.id.clone(), task.clone());
    Ok((StatusCode::CREATED, Json(task)))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Task>, ApiError> {
    let store = state.0.lock().expect("task store poisoned");
    store
        .tasks
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::not_found("task not found"))
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    payload: Result<Json<UpdateTask>, JsonRejection>,
) -> Result<Json<Task>, ApiError> {
    let body = json_body(payload)?;
    if body.title.is_none() && body.completed.is_none() {
        return Err(ApiError::bad_request("title or completed is required"));
    }
    let title = body.title.map(|title| title.trim().to_owned());
    if title.as_deref() == Some("") {
        return Err(ApiError::bad_request("title must be a non-empty string"));
    }

    let mut store = state.0.lock().expect("task store poisoned");
    let task = store
        .tasks
        .get_mut(&id)
        .ok_or_else(|| ApiError::not_found("task not found"))?;
    if let Some(title) = title {
        task.title = title;
    }
    if let Some(completed) = body.completed {
        task.completed = completed;
    }
    Ok(Json(task.clone()))
}

async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut store = state.0.lock().expect("task store poisoned");
    if store.tasks.remove(&id).is_none() {
        return Err(ApiError::not_found("task not found"));
    }
    Ok(StatusCode::NO_CONTENT)
}

fn json_body<T: DeserializeOwned>(payload: Result<Json<T>, JsonRejection>) -> Result<T, ApiError> {
    payload.map(|Json(body)| body).map_err(|rejection| {
        let message = if rejection.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE {
            "content-type must be application/json"
        } else if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
            "request body exceeds 1 MB"
        } else {
            "invalid JSON"
        };
        ApiError::bad_request(message)
    })
}

struct ApiError {
    status: StatusCode,
    message: &'static str,
}

impl ApiError {
    fn bad_request(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    fn not_found(message: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}
