use std::str::FromStr;
use tokio::sync::Mutex;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use p_hash::core::app;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow, sqlite::SqliteConnectOptions};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

type DynError = Box<dyn std::error::Error>;

#[actix_web::main]
async fn main() -> Result<(), DynError> {
    init_logger()?;
    let state = web::Data::new(State::try_new().await?);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .service(health)
            .service(runs)
            .service(get_app)
            .service(init_app)
            .service(get_hashing_methods)
            .service(get_modifications)
            .app_data(state.clone())
            .wrap(cors)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/runs")]
async fn runs(data: web::Data<State>) -> impl Responder {
    let runs = match data.get_runs().await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    HttpResponse::Ok().json(runs)
}

#[get("/app")]
async fn get_app(data: web::Data<State>) -> impl Responder {
    match &*data.app.lock().await {
        Some(a) => {
            let state = a.state();

            let hashing_methods = state.hashing_methods().iter().map(|h| h.name()).collect();
            let modifications = state
                .modifications()
                .iter()
                .map(|m| m.name().to_string())
                .collect();

            let data = AppData {
                hashing_methods,
                modifications,
            };
            HttpResponse::Ok().json(AppResponse {
                status: AppStatus::Initialized,
                data: Some(data),
            })
        }
        None => HttpResponse::Ok().json(AppResponse::default()),
    }
}

#[get("/app/init")]
async fn init_app(data: web::Data<State>) -> impl Responder {
    let mut app = data.app.lock().await;
    match *app {
        None => match app::App::try_default().await {
            Ok(a) => {
                *app = Some(a);
                return HttpResponse::Ok().json(InitStatus::Success);
            }
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        },
        Some(_) => HttpResponse::Conflict().json(InitStatus::AlreadyInitialized),
    }
}

#[get("/app/hashing_methods")]
async fn get_hashing_methods(data: web::Data<State>) -> impl Responder {
    let app = data.app.lock().await;

    match &*app {
        Some(a) => {
            let hashing_methods = a
                .state()
                .hashing_methods()
                .iter()
                .map(|h| h.name())
                .collect::<Vec<String>>();
            HttpResponse::Ok().json(hashing_methods)
        }
        None => HttpResponse::Conflict().json("app_unitialized"),
    }
}
#[get("/app/modifications")]
async fn get_modifications(data: web::Data<State>) -> impl Responder {
    let app = data.app.lock().await;

    match &*app {
        Some(a) => {
            let modifications = a
                .state()
                .hashing_methods()
                .iter()
                .map(|m| m.name().to_string())
                .collect::<Vec<String>>();
            HttpResponse::Ok().json(modifications)
        }
        None => HttpResponse::Conflict().json("app_unitialized"),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum InitStatus {
    Success,
    AlreadyInitialized,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum AppStatus {
    Initialized,
    #[default]
    Uninitialized,
}

#[derive(Serialize, Deserialize)]
struct AppData {
    hashing_methods: Vec<String>,
    modifications: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct AppResponse {
    status: AppStatus,
    data: Option<AppData>,
}

struct State {
    db: SqlitePool,
    app: Mutex<Option<app::App>>,
}
impl State {
    async fn get_runs(&self) -> Result<Runs, DynError> {
        let r: Vec<Run> = sqlx::query_as("SELECT * FROM runs")
            .fetch_all(&self.db)
            .await?;

        Ok(Runs { runs: r })
    }
}

impl State {
    pub async fn try_new() -> Result<Self, DynError> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite:data.db")?
                .create_if_missing(true)
                .pragma("cache_size", "200000"),
        )
        .await?;

        Ok(State {
            db: pool,
            app: Mutex::new(None),
        })
    }
}

#[derive(Serialize, Deserialize, FromRow)]
struct Run {
    id: u32,
    timestamp: u64,
}
#[derive(Serialize, Deserialize)]
struct Runs {
    runs: Vec<Run>,
}

fn init_logger() -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    Ok(())
}
