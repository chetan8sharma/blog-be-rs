use actix_web::{ web, post, Responder };
use sea_orm::{ Set, ActiveModelTrait };

use crate::utils::{api_response, app_state};
use crate::entity;

#[derive(serde::Deserialize)]
struct RegisterModel {
    name: String,
    email: String,
    password: String
}

#[post("/register")]
pub async fn register(
    app_state: web::Data<app_state::AppState>,
    register_json: web::Json<RegisterModel>
) -> impl Responder {
    let user_model = entity::user::ActiveModel {
        name: Set(register_json.name.clone()),
        email: Set(register_json.email.clone()),
        password: Set(register_json.password.clone()),
        ..Default::default()
    }.insert(&app_state.db).await.unwrap();

    api_response::ApiResponse::new(200, format!("{}", user_model.id))
}


#[post("/login")]
pub async fn login(
    app_state: web::Data<app_state::AppState>,
    register_json: web::Json<RegisterModel>
) -> impl Responder {
    let user_model = entity::user::ActiveModel {
        name: Set(register_json.name.clone()),
        email: Set(register_json.email.clone()),
        password: Set(register_json.password.clone()),
        ..Default::default()
    }.insert(&app_state.db).await.unwrap();

    api_response::ApiResponse::new(200, format!("{}", user_model.id))
}
