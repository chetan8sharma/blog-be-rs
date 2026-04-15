use actix_web::{ web, post, get };
use std::path::PathBuf;
use crate::utils::{api_response::ApiResponse, app_state, jwt::Claims};
use serde::{Serialize, Deserialize};
use crate::entity;
use sea_orm::{Set, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait };
use uuid::Uuid;
use chrono::{ Utc, NaiveDateTime };
use actix_multipart::form::{ MultipartForm, text::Text, tempfile::TempFile };
use crate::utils;

#[derive(MultipartForm)]
struct CreatePostModel {
    title: Text<String>,
    text: Text<String>,
    file: TempFile
}

#[derive(Serialize, Deserialize)]
struct PostModel {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub uuid: Uuid,
    pub image: Option<String>,
    pub user_id: i32,
    pub created_at: NaiveDateTime,
    pub user: Option<UserModel>
}

#[derive(Serialize, Deserialize)]
struct UserModel {
    name: String,
    email: String
}

#[post("create")]
pub async fn create_post(
    app_state: web::Data<app_state::AppState>,
    claim: Claims,
    post_model: MultipartForm<CreatePostModel>
) -> Result<ApiResponse, ApiResponse> {

    let check_name = post_model.file.file_name.clone().unwrap_or("null".to_owned());
    let max_file_size = (*utils::constants::MAX_FILE_SIZE).clone();

    match &check_name[check_name.len() - 4 ..] {
        ".png" | ".jpg" => {},
        _ => {
            return Err(ApiResponse::new(400, "invalid file type".to_string()))
        }
    }

    match post_model.file.size {
        0 => {
            return Err(ApiResponse::new(400, "invalid file type".to_string()))
        },
        length if length > max_file_size as usize => {
            return Err(ApiResponse::new(400, "file too big".to_string()))
        },
        _ => {}
    }

    let txn = app_state.db.begin().await
    .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    let post_entity = entity::post::ActiveModel {
        title: Set(post_model.title.clone()),
        text: Set(post_model.text.clone()),
        uuid: Set(Uuid::new_v4()),
        user_id: Set(claim.id),
        created_at: Set(Utc::now().naive_local()),
        ..Default::default()
    };

    let mut created_entity = post_entity.save(&txn).await
    .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    let temp_file_path = post_model.file.file.path();
    let file_name = post_model.file.file_name.as_ref()
    .map(|m| m.as_ref())
    .unwrap_or("null");

    let time_stamp: i64 = Utc::now().timestamp();

    let mut file_path = PathBuf::from("./public");
    let new_file_name = format!("{}-{}", time_stamp, file_name);
    file_path.push(&new_file_name);

    let _ = match std::fs::copy(temp_file_path, file_path) {
        Ok(_) => {
            created_entity.image = Set(Some(new_file_name));
            created_entity.save(&txn).await
                .map_err(|err| ApiResponse::new(500, err.to_string()))?;

            txn.commit().await
                .map_err(|err| ApiResponse::new(500, err.to_string()))?;


            std::fs::remove_file(temp_file_path).unwrap_or_default();
            Ok(ApiResponse::new(200, "file uploaded".to_owned()))
        }
        Err(_) => {
            txn.rollback().await
                .map_err(|err| ApiResponse::new(500, err.to_string()))?;

            Err(ApiResponse::new(500, "Internal Server Error".to_string()))
        }
    };

    Ok(ApiResponse::new(200, "post created".to_string()))
}

#[get("my-posts")]
pub async fn my_posts(
    app_state: web::Data<app_state::AppState>,
    claim: Claims,
) -> Result<ApiResponse, ApiResponse> {
    let posts: Vec<PostModel> = entity::post::Entity::find()
        .filter(entity::post::Column::UserId.eq(claim.id)).all(&app_state.db).await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?
        .into_iter()
        .map(|post| 
            PostModel {
                id: post.id,
                title: post.title,
                text: post.text,
                uuid: post.uuid,
                image: post.image,
                user_id: post.user_id,
                created_at: post.created_at,
                user: None
            }
        ).collect();

    let res_str = serde_json::to_string(&posts)
    .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    Ok(ApiResponse::new(200, res_str.to_owned()))
}

#[get("all-posts")]
pub async fn all_posts(
    app_state: web::Data<app_state::AppState>,
) -> Result<ApiResponse, ApiResponse> {
    let posts: Vec<PostModel> = entity::post::Entity::find()
        .all(&app_state.db).await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?
        .into_iter()
        .map(|post| 
            PostModel {
                id: post.id,
                title: post.title,
                text: post.text,
                uuid: post.uuid,
                image: post.image,
                user_id: post.user_id,
                created_at: post.created_at,
                user: None
            }
        ).collect();

    let res_str = serde_json::to_string(&posts)
    .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    Ok(ApiResponse::new(200, res_str.to_owned()))
}

#[get("one-post/{post_uuid}")]
pub async fn one_post(
    app_state: web::Data<app_state::AppState>,
    post_uuid: web::Path<Uuid>
) -> Result<ApiResponse, ApiResponse> {
    let posts: PostModel = entity::post::Entity::find()
        .filter(entity::post::Column::Uuid.eq(post_uuid.clone()))
        .find_also_related(entity::user::Entity)
        .one(&app_state.db).await
        .map_err(|err| ApiResponse::new(500, err.to_string()))?
        .map(|post| 
            PostModel {
                id: post.0.id,
                title: post.0.title,
                text: post.0.text,
                uuid: post.0.uuid,
                image: post.0.image,
                user_id: post.0.user_id,
                created_at: post.0.created_at,
                user: post.1.map(|item| UserModel { name: item.name, email: item.email })
            }
        ).ok_or(ApiResponse::new(404, "no post found".to_string()))?;

    let res_str = serde_json::to_string(&posts)
    .map_err(|err| ApiResponse::new(500, err.to_string()))?;

    Ok(ApiResponse::new(200, res_str.to_owned()))
}
