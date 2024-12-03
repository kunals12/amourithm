pub mod auth;
pub use auth::Register;
use serde::Serialize;

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    success: bool,
    message: String,
    data: Option<T>,
}
