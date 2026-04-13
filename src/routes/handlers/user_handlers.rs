use actix_web::{ get, Responder };

use crate::utils::{ api_response };

#[get("")]
pub async fn user() -> impl Responder {
    api_response::ApiResponse::new(200, "Verified user".to_string())
}
