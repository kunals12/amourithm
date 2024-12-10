use actix_web::{
    web::{get, patch, post, Data},
    App, HttpResponse, HttpServer, Responder,
};
mod auth;
use auth::Register;
use database::database_connection;
mod database;
mod user;
use user::User;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let database = database_connection()
        .await
        .expect("Failed to connect to database");
    println!("Database Connection Established");
    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(database.clone()))
            .route("/", get().to(hello_world))
            // Auth Routes
            .route("/api/v1/auth/signup", post().to(Register::register_user))
            .route("/api/v1/auth/signin", post().to(Register::login_user))
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
