use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Router,
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use sipp_rust::db::{self, Db};
use sipp_rust::highlight::Highlighter;
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    db: Db,
    highlighter: Arc<Highlighter>,
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

#[tokio::main]
async fn main() {
    let state = AppState {
        db: db::init_db(),
        highlighter: Arc::new(Highlighter::new()),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/about", get(about))
        .route("/s/{short_id}", get(view_snippet))
        .route("/snippets", post(create_snippet))
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
