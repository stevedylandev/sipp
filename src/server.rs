use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Json, Router,
    extract::{Form, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use rust_embed::Embed;
use serde::Deserialize;
use crate::db::{self, Db, Snippet};
use crate::highlight::Highlighter;
use std::sync::Arc;

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

#[derive(Embed)]
#[folder = "static/"]
struct Static;

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

fn mime_from_path(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "css" => "text/css",
        "js" => "application/javascript",
        "html" => "text/html",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "svg" => "image/svg+xml",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "json" | "webmanifest" => "application/json",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

async fn serve_assets(Path(path): Path<String>) -> Response {
    match Assets::get(&path) {
        Some(file) => {
            let mime = mime_from_path(&path);
            ([(header::CONTENT_TYPE, mime)], file.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn serve_static(Path(path): Path<String>) -> Response {
    match Static::get(&path) {
        Some(file) => {
            let mime = mime_from_path(&path);
            ([(header::CONTENT_TYPE, mime)], file.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn run(host: String, port: u16) {
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
        .route("/assets/{*path}", get(serve_assets))
        .route("/static/{*path}", get(serve_static))
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {}", addr));

    println!("Server running at http://{}:{}", host, port);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
