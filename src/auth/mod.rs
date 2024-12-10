pub mod auth;
pub use auth::Register;
use serde::Serialize;
pub mod jwt;
pub mod utils;

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}
