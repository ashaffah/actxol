#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
mod model;

use model::User;
use std::{ convert::Infallible, io };
use actix_cors::Cors;
use actix_files::{ Files, NamedFile };
use actix_session::{ storage::CookieSessionStore, Session, SessionMiddleware };
use actix_web::{
    error,
    get,
    http::{ header::{ self, ContentType }, Method, StatusCode },
    middleware::{ Logger, Compress },
    post,
    web::{ self, Json },
    App,
    Either,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
    Result,
};
use mongodb::{ bson::doc, options::IndexOptions, Client, Collection, Database, IndexModel };
use async_stream::stream;
use qirust::helper::generate_svg_string;
use serde::Deserialize;

// NOTE: Not a suitable session key for production.
static SESSION_SIGNING_KEY: &[u8] = &[0; 64];

/// favicon handler
#[get("/favicon")]
async fn favicon() -> Result<impl Responder> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

/// simple index handler
#[get("/welcome")]
async fn welcome(req: HttpRequest, session: Session) -> Result<HttpResponse> {
    println!("{req:?}");

    // session
    let mut counter = 1;
    if let Some(count) = session.get::<i32>("counter")? {
        println!("SESSION value: {count}");
        counter = count + 1;
    }

    // set counter to session
    session.insert("counter", counter)?;

    // response
    Ok(
        HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::html())
            .body(include_str!("../static/welcome.html"))
    )
}

#[derive(Deserialize)]
struct Info {
    data: String,
}
#[derive(Deserialize)]
struct DataJson {
    message: String,
    data: String,
}

// this handler gets called if the query deserializes into `Info` successfully
// otherwise a 400 Bad Request error response is returned
#[get("/api")]
async fn index(info: web::Query<Info>) -> Result<HttpResponse> {
    let svg_string = generate_svg_string(&info.data);
    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::html()).body(svg_string))
}

#[post("/api/svg")]
async fn get_svg(info: web::Json<Info>) -> Result<HttpResponse> {
    let svg_string = generate_svg_string(&info.data);

    Ok(HttpResponse::build(StatusCode::OK).content_type(ContentType::json()).body(svg_string))
}

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

/// Adds a new user to the "users" collection in the database.
#[post("/api/add_user")]
async fn add_user(cfg: web::Data<AppStates>, json: web::Json<User>) -> HttpResponse {
    let collection: Collection<User> = cfg.db.collection("users");
    let result = collection.insert_one(json.into_inner()).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("user added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Gets the user with the supplied username.
#[get("/api/get_user/{username}")]
async fn get_user(cfg: web::Data<AppStates>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = cfg.db.collection("users");
    match collection.find_one(doc! { "username": &username }).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
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

// async fn indexx(data: web::Data<AppState>) {
//     let postgres_conn_str = data.postgres_db.get_connection_string();
//     format!("PostgreSQL: {}", postgres_conn_str);
// }

struct AppStates {
    pub db: Database,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // random key means that restarting server will invalidate existing session cookies
    let key = actix_web::cookie::Key::from(SESSION_SIGNING_KEY);

    // log::info!("starting HTTP server at http://localhost:8080");

    let bind_addr = "0.0.0.0:8080";
    // println!("Server Running at {} ....", bind_addr);

    let uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    let db = client.database(&std::env::var("DB_NAME").unwrap_or_else(|_| "myApp".into()));
    create_username_index(&client).await;

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
                    db: db.clone(),
                })
            )
            .wrap(cors)
            // enable automatic response compression - usually register this first
            .wrap(Compress::default())
            // cookie session middleware
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(false)
                    .build()
            )
            // enable logger - always register Actix Web Logger middleware last
            .wrap(Logger::default())
            // register favicon
            .service(favicon)
            // register simple route, handle all methods
            .service(welcome)
            .service(get_svg)
            .service(index)
            .service(add_user)
            .service(get_user)
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
        .bind(bind_addr)?
        .workers(2)
        .run().await
}
