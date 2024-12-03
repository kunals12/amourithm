use actix_web::{
    web::{get, post},
    App, HttpResponse, HttpServer, Responder,
};
mod auth;
use auth::Register;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", get().to(hello_world))
            .route("/api/v1/auth/signup", post().to(Register::register_user))
            .route("/api/v1/auth/signin", post().to(Register::login_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn hello_world() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}
