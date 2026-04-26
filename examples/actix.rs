use std::{env::home_dir, fs::{File, create_dir, create_dir_all}, io::copy, path::{Path, PathBuf}, str::FromStr};

use actix_cors::Cors;
use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use p_hash::{core::app, db};
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
            .service(get_hashing_methods)
            .service(get_modifications)
            .service(get_run_hashing_methods)
            .service(get_run_modifications)
            .service(get_modifications)
            .service(submit_image)
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

/// Hashing methods for the current run setup
#[get("/run/hashing_methods")]
async fn get_run_hashing_methods(data: web::Data<State>) -> impl Responder {
    let hashing_methods = data.app.state().get_run_hashes();
    HttpResponse::Ok().json(hashing_methods)
}
#[get("/run/modifications")]
async fn get_run_modifications(data: web::Data<State>) -> impl Responder {
    let hashing_methods = data.app.state().get_run_modifications();
    HttpResponse::Ok().json(hashing_methods)
}

#[get("/hashing_methods")]
async fn get_hashing_methods(data: web::Data<State>) -> impl Responder {
    let methods: Vec<String> = data
        .app
        .state()
        .hashing_methods()
        .iter()
        .map(|m| m.name().to_string())
        .collect();
    HttpResponse::Ok().json(methods)
}

#[get("/modifications")]
async fn get_modifications(data: web::Data<State>) -> impl Responder {
    let methods: Vec<String> = data
        .app
        .state()
        .modifications()
        .iter()
        .map(|m| m.name().to_string())
        .collect();
    HttpResponse::Ok().json(methods)
}

#[post("/run/start")]
async fn start_run(data: web::Data<State>, config: web::Json<AppConfig>) -> impl Responder {
    let mod_ids = config.modifications.iter().map(|i| *i as usize).collect();
    data.app.set_selected_modifications(mod_ids);

    let hash_ids = config.hashing_methods.iter().map(|i| *i as usize).collect();
    data.app.set_selected_hashing_methods(hash_ids);

    data.app.set_path(config.path.clone());

    if let Err(e) = data.app.run().await {
        return HttpResponse::InternalServerError().json(e.to_string());
    };

    HttpResponse::Ok().into()
}

#[post("/images/submit")]
async fn submit_image(data: web::Data<State>, MultipartForm(form): MultipartForm<UploadForm>)-> impl Responder{
    tracing::debug!("Inserting images for user {}", form.username.to_string());

    let mut tx = match data.db.begin().await{
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().json(e.to_string())
    };

    let mut skipped = 0;

    for f in form.files{
        let mut file_name = match f.file_name {
            Some(f) => PathBuf::from(f),
            None => {tracing::warn!("Image did not have filename"); skipped+=1;continue}
        };
        let mut save_path = data.image_save_path.clone();

        // Holy refactor please
        if file_name.parent().is_some(){
            file_name = PathBuf::from(file_name.file_name().unwrap().to_str().unwrap());
        }

        save_path.push(file_name);

        let mut src = f.file;
        let mut dst = match File::create(&save_path){
            Ok(f) => f,
            Err(e) => {tracing::error!("{}", e.to_string());return HttpResponse::InternalServerError().json(e.to_string());}
        };
        if let Err(e ) = copy(&mut src, &mut dst){
            tracing::error!("{}", e.to_string());
            return HttpResponse::InternalServerError().json(e.to_string());
        };

        let res = sqlx::query(
            "
        INSERT INTO images (path, user) VALUES (?, ?) ON CONFLICT(id) DO NOTHING;
        ",
        )
        .bind(save_path.to_str().unwrap())
        .bind(&form.username.to_string())
        .execute(&mut *tx)
        .await;
        if let Err(e) = res { return HttpResponse::InternalServerError().json(e.to_string())}
        }
    if let Err (e) = tx.commit().await{
        return HttpResponse::InternalServerError().json(e.to_string())
    };
    HttpResponse::Ok().json(format!("{{skipped: {}}}", skipped))
}

#[derive(Debug, MultipartForm)]
struct UploadForm{
    username: Text<String>,
    #[multipart(rename = "images")]
    files: Vec<TempFile>
}

/// Submit form to initialize app
#[derive(Serialize, Deserialize)]
struct AppConfig {
    path: PathBuf,
    hashing_methods: Vec<u32>,
    modifications: Vec<u32>,
}

struct State {
    image_save_path: PathBuf,
    db: SqlitePool,
    app: app::App,
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

        let app = app::App::try_default().await?;
        let mut image_save_path = home_dir().unwrap();
        image_save_path.push(".local/share/p-hash/images");

        create_dir_all(&image_save_path)?;

        Ok(State { db: pool, app , image_save_path: image_save_path.to_path_buf()})
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
