pub mod auth;
pub use auth::Register;
use serde::Serialize;
pub mod jwt;
pub mod utils;

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}
