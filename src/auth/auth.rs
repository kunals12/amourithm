use super::SuccessResponse;
use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Register {
    email: String,
    password: String,
}

impl Register {
    pub async fn register_user() -> impl Responder {
        HttpResponse::Created().json(SuccessResponse::<()> {
            success: true,
            message: "Email Sent Successfully".to_string(),
            data: None,
        })
    }

    pub async fn login_user() -> impl Responder {
        HttpResponse::Ok().json(SuccessResponse {
            success: true,
            message: "User Logged-In".to_string(),
            data: Some(()),
        })
    }
}
