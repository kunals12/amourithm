use actix_web::HttpResponse;
use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseToSend<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

pub fn handle_bad_request(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(ResponseToSend::<()> {
        success: false,
        message: message.to_string(),
        data: None,
    })
}

pub fn handle_internal_server_error(message: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(ResponseToSend::<()> {
        success: false,
        message: message.to_string(),
        data: None,
    })
}

pub fn handle_conflict_error(message: &str) -> HttpResponse {
    HttpResponse::Conflict().json(ResponseToSend::<()> {
        success: true,
        message: message.to_string(),
        data: None,
    })
}

pub fn handle_not_found_error(message: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ResponseToSend::<()> {
        success: true,
        message: message.to_string(),
        data: None,
    })
}
