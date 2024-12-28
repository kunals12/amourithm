use std::sync::Arc;
use actix_web::{
    web::{get, patch, post, Data},
    App, HttpResponse, HttpServer, Responder,
};
mod auth;
use auth::Register;
mod connections;
use connections::*;
mod user;
use ::redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;
use sqlx::{Pool, Postgres};
use user::User;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let database: Pool<Postgres> = database_connection()
        .await
        .expect("Failed to connect to database");
    println!("Database Connection Established");
    let redis: Arc<Mutex<MultiplexedConnection>> = connect_to_redis().await.expect("Failed to connect to redis");
    // Share RedisService with the app
    let redis_service_data = Data::new(redis);
    
    println!("Redis Connection Established");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(database.clone()))
            .app_data(redis_service_data.clone())
            .route("/", get().to(hello_world))
            // Auth Routes
            .route("/api/v1/auth/signup", post().to(Register::register_user))
            .route("/api/v1/auth/signin", post().to(Register::login_user))
            .route("/api/v1/auth/verify-otp", post().to(Register::verify_otp))
            .route("/api/v1/auth/user", get().to(Register::get_user))
            // User Routes
            .route("/api/v1/user/update", patch().to(User::update_user_details))
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    server.await
}

async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}
