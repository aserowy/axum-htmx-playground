use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Form, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/", get(home))
        .route("/entries", get(get_entries).post(post_entry))
        .route("/entries/:id", delete(delete_entry));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate;

async fn home() -> impl IntoResponse {
    HomeTemplate
}

#[derive(Template)]
#[template(path = "entries.html")]
struct EntriesTemplate {
    entries: Vec<Entry>,
}

struct Entry {
    id: Uuid,
    content: String,
}

async fn get_entries() -> impl IntoResponse {
    sleep(Duration::from_secs(3)).await;

    EntriesTemplate {
        entries: vec![
            Entry {
                id: Uuid::new_v4(),
                content: "content...".to_string(),
            },
            Entry {
                id: Uuid::new_v4(),
                content: "more content...".to_string(),
            },
        ],
    }
}

#[derive(Serialize, Deserialize)]
struct EntryForm {
    content: String,
}

#[derive(Template)]
#[template(path = "entry.html")]
struct EntryTemplate {
    entry: Entry,
}

async fn post_entry(Form(entry): Form<EntryForm>) -> impl IntoResponse {
    println!("creating entry with content: {}", entry.content);

    EntryTemplate {
        entry: Entry {
            id: Uuid::new_v4(),
            content: entry.content,
        },
    }
}

async fn delete_entry(Path(id): Path<String>) -> impl IntoResponse {
    let uuid = Uuid::parse_str(&id).unwrap();

    println!("deleting entry with id {}", uuid);

    StatusCode::OK
}
