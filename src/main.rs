// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
mod configs;
mod models;
mod handlers;

use std::{ convert::Infallible, env, io };
use actix_cors::Cors;
use actix_files::{ Files, NamedFile };
use actix_session::{ storage::CookieSessionStore, SessionMiddleware };
use actix_web::{
    error::{ self, Error, InternalError, JsonPayloadError },
    http::{ header::{ self, ContentType }, Method, StatusCode },
    middleware::{ Compress, Logger },
    web::{ self, scope, JsonConfig },
    App,
    Either,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
    Result,
};
use chrono::{ SecondsFormat, Utc };
use dotenvy::dotenv;
use log::info;
use models::{ error_model::ApiError, user_model::User };
use handlers::{
    qr_handler::{ generate_qr, get_svg },
    user_handler::{ add_user, get_user },
    welcome_handler::{ favicon, welcome },
};
use mongodb::{ bson::doc, options::IndexOptions, Client, IndexModel };
use configs::db::{ init, AppStates };
use async_stream::stream;

// NOTE: Not a suitable session key for production.
static SESSION_SIGNING_KEY: &[u8] = &[0; 64];

async fn default_handler(req_method: Method) -> Result<impl Responder> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open("static/404.html")?
                .customize()
                .with_status(StatusCode::NOT_FOUND);
            Ok(Either::Left(file))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

async fn streaming_response(path: web::Path<String>) -> HttpResponse {
    let name = path.into_inner();

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .streaming(
            stream! {
            yield Ok::<_, Infallible>(web::Bytes::from("Hello "));
            yield Ok::<_, Infallible>(web::Bytes::from(name));
            yield Ok::<_, Infallible>(web::Bytes::from("!"));
        }
        )
}

/// Creates an index on the "username" field to force the values to be unique.
async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    client
        .database(&std::env::var("DB_NAME").unwrap_or_else(|_| "myApp".into()))
        .collection::<User>(&std::env::var("COLL_NAME").unwrap_or_else(|_| "users".into()))
        .create_index(model).await
        .expect("creating an index should succeed");
}

// Handle json parser errors.
fn json_error_handler(err: JsonPayloadError, _req: &HttpRequest) -> Error {
    let detail = err.to_string();
    let resp = match &err {
        JsonPayloadError::ContentType =>
            HttpResponse::UnsupportedMediaType().json(ApiError {
                status: 415,
                time: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
                message: "Unsupported media type".to_owned(),
                debug_message: Some(detail),
                sub_errors: Vec::new(),
            }),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => {
            HttpResponse::UnprocessableEntity().json(ApiError {
                status: 422,
                time: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
                message: "Unprocessable payload".to_owned(),
                debug_message: Some(detail),
                sub_errors: Vec::new(),
            })
        }
        _ =>
            HttpResponse::BadRequest().json(ApiError {
                status: 400,
                time: Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true),
                message: "Bad request. Missing parameter and / or wrong payload.".to_owned(),
                debug_message: Some(detail),
                sub_errors: Vec::new(),
            }),
    };
    InternalError::from_response(err, resp).into()
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // random key means that restarting server will invalidate existing session cookies
    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);
    // Load .env file
    dotenv().ok();
    // Initialize MongoDB connection
    let client = init().await;
    let db = client.database(&env::var("DB_NAME").unwrap_or_else(|_| "myApp".into()));
    create_username_index(&client).await;
    // let db_postgres = connect().await;

    // Get Server host and port number from environment file.
    let server_host = match env::var("SERVER.HOST") {
        Ok(v) => v.to_string(),
        Err(_) => "127.0.0.1".to_string(),
    };

    let server_port: u16 = match env::var("SERVER.PORT") {
        Ok(v) => v.parse().unwrap_or(8080),
        Err(_) => 8080,
    };
    info!("Starting actix-web server in {}:{}", server_host, server_port);

    HttpServer::new(move || {
        // CORS
        let cors = Cors::default()
            // .allowed_origin("https://www.rust-lang.org")
            // .allowed_origin_fn(|origin, _req_head| {
            //     origin.as_bytes().ends_with(b".rust-lang.org")
            // })
            .send_wildcard()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .app_data(
                web::Data::new(AppStates {
                    // postgres_db: db_postgres,
                    // client_mongo: client.clone(),
                    mongo_db: db.clone(),
                })
            )
            .app_data(JsonConfig::default().error_handler(json_error_handler))
            .wrap(cors)
            // enable automatic response compression - usually register this first
            .wrap(Compress::default())
            // cookie session middleware
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(false)
                    .build()
            )
            .service(
                // Prefix route
                scope("/api")
                    .service(get_user)
                    .service(get_svg)
                    .service(add_user)
                    .service(generate_qr)
            )
            // enable logger - always register Actix Web Logger middleware last
            .wrap(Logger::default())
            // register favicon
            .service(favicon)
            // register simple route, handle all methods
            .service(welcome)
            // with path parameters
            // async response body
            .service(web::resource("/async-body/{name}").route(web::get().to(streaming_response)))
            .service(
                web::resource("/test").to(|req: HttpRequest| {
                    match *req.method() {
                        Method::GET => HttpResponse::Ok(),
                        Method::POST => HttpResponse::MethodNotAllowed(),
                        _ => HttpResponse::NotFound(),
                    }
                })
            )
            .service(
                web
                    ::resource("/error")
                    .to(|| async {
                        error::InternalError::new(
                            io::Error::new(io::ErrorKind::Other, "test"),
                            StatusCode::INTERNAL_SERVER_ERROR
                        )
                    })
            )
            // static files
            .service(Files::new("/static", "static").show_files_listing())
            // redirect
            .service(
                web::resource("/").route(
                    web::get().to(|req: HttpRequest| async move {
                        println!("{req:?}");
                        HttpResponse::Found()
                            .insert_header((header::LOCATION, "static/welcome.html"))
                            .finish()
                    })
                )
            )
            // default
            .default_service(web::to(default_handler))
    })
        .bind((server_host, server_port))?
        .workers(2)
        .run().await
}
