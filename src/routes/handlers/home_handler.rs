use actix_web::{web, Responder, get};
use sea_orm::{ConnectionTrait, Statement};

use crate::utils::{ api_response::ApiResponse, app_state::AppState};

#[get("/hello/{name}")]
pub async fn greet(name: web::Path<String>) -> impl Responder {
    ApiResponse::new(200, format!("Hello {name}"))
}

#[get("/test")]
pub async fn test(app_state: web::Data<AppState>) -> Result<ApiResponse, ApiResponse> {
    let res = app_state.db
        .query_all_raw(Statement::from_string(sea_orm::DatabaseBackend::Postgres, "SELECT * FROM USER;"))
        .await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    Ok(ApiResponse::new(200, "test".to_string()))
}
