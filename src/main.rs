use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{delete, get},
    Extension, Form, Router,
};
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, fmt, net::SocketAddr, path::PathBuf};
use tokio::{
    sync::broadcast::{channel, Sender},
    time::{sleep, Duration},
};
use tokio_stream::{wrappers::BroadcastStream, Stream, StreamExt as _};
use tower_http::services::ServeDir;
use uuid::Uuid;

pub type NotificationSender = Sender<NotificationTemplate>;

#[derive(Clone, Serialize)]
pub enum Severity {
    Success,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Severity::Success => write!(f, "success"),
            Severity::Error => write!(f, "error"),
        }
    }
}

#[derive(Clone, Template)]
#[template(path = "notification.html")]
pub struct NotificationTemplate {
    pub id: String,
    pub severity: Severity,
    pub message: String,
}

#[tokio::main]
async fn main() {
    let (notification_sender, _) = channel::<NotificationTemplate>(10);

    let router = Router::new()
        .route("/", get(home))
        .route("/entries", get(get_entries).post(post_entry))
        .route("/entries/:id", delete(delete_entry))
        .route("/notifications", get(get_notification_sse))
        .nest_service("/assets", ServeDir::new(PathBuf::from("assets")))
        .layer(Extension(notification_sender));

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
    sleep(Duration::from_secs(1)).await;

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

#[derive(Deserialize, Serialize)]
struct EntryForm {
    content: String,
}

#[derive(Template)]
#[template(path = "entry.html")]
struct EntryTemplate {
    entry: Entry,
}

async fn post_entry(
    Extension(sender): Extension<NotificationSender>,
    Form(entry): Form<EntryForm>,
) -> impl IntoResponse {
    if let Err(_) = sender.send(NotificationTemplate {
        id: Uuid::new_v4().to_string(),
        severity: Severity::Success,
        message: format!("created entry with content: {}", entry.content),
    }) {
        eprintln!("failed to send notification");
    }

    EntryTemplate {
        entry: Entry {
            id: Uuid::new_v4(),
            content: entry.content,
        },
    }
}

async fn delete_entry(
    Extension(sender): Extension<NotificationSender>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let uuid = Uuid::parse_str(&id).unwrap();

    if let Err(_) = sender.send(NotificationTemplate {
        id: Uuid::new_v4().to_string(),
        severity: Severity::Error,
        message: format!("deleted entry with id: {}", uuid),
    }) {
        eprintln!("failed to send notification");
    }

    StatusCode::OK
}

async fn get_notification_sse(
    Extension(sender): Extension<NotificationSender>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let subscriber = sender.subscribe();
    let stream = BroadcastStream::new(subscriber);

    Sse::new(
        stream
            .map(|ntfctn| {
                if let Ok(notification) = ntfctn {
                    let message = notification.render().unwrap();
                    Event::default().data(message)
                } else {
                    let message = "<div>error handling notification</div>".to_string();
                    Event::default().data(message)
                }
            })
            .map(Ok),
    )
    .keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(5))
            .text("keep-alive-message"),
    )
}
