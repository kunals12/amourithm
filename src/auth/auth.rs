use super::{
    jwt::generate_token,
    utils::{decrypt_password, encrypt_password},
    SuccessResponse,
};
use actix_web::{
    cookie::{time::Duration, Cookie, SameSite},
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Register {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct Login {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    id: uuid::Uuid,
    password: String,
}

impl Register {
    async fn check_user_existance(db: Data<PgPool>, username: &str) -> bool {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(username)
            .fetch_one(&**db)
            .await
            .unwrap_or(false)
    }

    pub async fn register_user(db: Data<PgPool>, user: Json<Register>) -> impl Responder {
        let is_user_exists = Self::check_user_existance(db.clone(), &user.username).await;

        if is_user_exists {
            return HttpResponse::Ok().json(SuccessResponse::<()> {
                success: true,
                message: "User Already Exists".to_string(),
                data: None,
            });
        }

        let user_id = Uuid::new_v4();

        let hash_password = encrypt_password(&user.password);

        let user = sqlx::query(
            "INSERT INTO users (id, username, email, password) VALUES ($1, $2, $3, $4)",
        )
        .bind(user_id)
        .bind(user.username.clone())
        .bind(user.email.clone())
        .bind(hash_password)
        .execute(&**db)
        .await;

        match user {
            Ok(_) => HttpResponse::Created().json(SuccessResponse::<()> {
                success: true,
                message: "Email Sent Successfully".to_string(),
                data: None,
            }),
            Err(e) => HttpResponse::InternalServerError().json(SuccessResponse::<()> {
                success: false,
                message: e.to_string(),
                data: None,
            }),
        }
    }

    // Checks the type of variable
    // fn type_of<T>(_: &T) -> &'static str {
    //     type_name::<T>()
    // }

    pub async fn login_user(db: Data<PgPool>, body: Json<Login>) -> impl Responder {
        let response =
            sqlx::query_as::<_, User>("SELECT id, password FROM users WHERE username = $1")
                .bind(&body.username)
                .fetch_one(&**db)
                .await;

        match response {
            Ok(user) => {
                let is_password_match = decrypt_password(&body.password.clone(), &user.password);

                if !is_password_match {
                    return HttpResponse::BadRequest().json(SuccessResponse::<()> {
                        success: false,
                        message: "Password Not Matched".to_string(),
                        data: None,
                    });
                }

                let token = generate_token(user.id);

                let cookie = Cookie::build("auth_token", &token)
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .max_age(Duration::hours(24))
                    .same_site(SameSite::Strict)
                    .finish();

                HttpResponse::Ok().cookie(cookie).json(SuccessResponse {
                    success: true,
                    message: "Signin Successfully".to_string(),
                    data: Some(token),
                })
            }
            Err(e) => {
                println!("{}", e);
                HttpResponse::NotFound().json(SuccessResponse::<()> {
                    success: false,
                    message: e.to_string(),
                    data: None,
                })
            }
        }
    }
}
