use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Json, Router,
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use sipp_rust::db::{self, Db, Snippet};
use sipp_rust::highlight::Highlighter;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    db: Db,
    highlighter: Arc<Highlighter>,
    api_key: Option<String>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "snippet.html")]
struct SnippetTemplate {
    name: String,
    content: String,
    highlighted_content: String,
}

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate;

#[derive(Deserialize)]
struct CreateSnippetForm {
    name: String,
    content: String,
}

async fn index() -> WebTemplate<IndexTemplate> {
    WebTemplate(IndexTemplate)
}

async fn about() -> WebTemplate<AboutTemplate> {
    WebTemplate(AboutTemplate)
}

async fn view_snippet(
    State(state): State<AppState>,
    Path(short_id): Path<String>,
) -> Result<WebTemplate<SnippetTemplate>, (StatusCode, Html<String>)> {
    match db::get_snippet_by_short_id(&state.db, &short_id) {
        Some(snippet) => {
            let highlighted_content = state.highlighter.highlight(&snippet.name, &snippet.content);
            Ok(WebTemplate(SnippetTemplate {
                name: snippet.name,
                content: snippet.content,
                highlighted_content,
            }))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Html("<h1>Snippet not found</h1>".to_string()),
        )),
    }
}

async fn create_snippet(
    State(state): State<AppState>,
    Form(form): Form<CreateSnippetForm>,
) -> impl IntoResponse {
    let snippet = db::create_snippet(&state.db, &form.name, &form.content);
    Redirect::to(&format!("/s/{}", snippet.short_id))
}

fn check_api_key(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let server_key = match &state.api_key {
        Some(k) => k,
        None => return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "No API key configured on server"})))),
    };
    let provided = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());
    match provided {
        Some(k) if k == server_key => Ok(()),
        _ => Err((StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Invalid or missing API key"})))),
    }
}

async fn api_list_snippets(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Snippet>>, (StatusCode, Json<serde_json::Value>)> {
    check_api_key(&state, &headers)?;
    Ok(Json(db::get_all_snippets(&state.db)))
}

async fn api_get_snippet(
    State(state): State<AppState>,
    Path(short_id): Path<String>,
) -> Result<Json<Snippet>, (StatusCode, Json<serde_json::Value>)> {
    match db::get_snippet_by_short_id(&state.db, &short_id) {
        Some(snippet) => Ok(Json(snippet)),
        None => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Snippet not found"})))),
    }
}

#[derive(Deserialize)]
struct ApiCreateSnippet {
    name: String,
    content: String,
}

async fn api_create_snippet(
    State(state): State<AppState>,
    Json(body): Json<ApiCreateSnippet>,
) -> (StatusCode, Json<Snippet>) {
    let snippet = db::create_snippet(&state.db, &body.name, &body.content);
    (StatusCode::CREATED, Json(snippet))
}

async fn api_delete_snippet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(short_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    check_api_key(&state, &headers)?;
    if db::delete_snippet_by_short_id(&state.db, &short_id) {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Snippet not found"}))))
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        db: db::init_db(),
        highlighter: Arc::new(Highlighter::new()),
        api_key: std::env::var("SIPP_API_KEY").ok(),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/about", get(about))
        .route("/s/{short_id}", get(view_snippet))
        .route("/snippets", post(create_snippet))
        .route("/api/snippets", get(api_list_snippets).post(api_create_snippet))
        .route("/api/snippets/{short_id}", get(api_get_snippet).delete(api_delete_snippet))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Server running at http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
