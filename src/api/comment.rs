use crate::model::comment::{Comment, NewComment};
use crate::repo::comment as comment_repo;
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpResponse, Result,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[post("/posts/{post_id}/comments")]
pub async fn create_comment(
    pool: Data<PgPool>,
    path: Path<Uuid>,
    body: Json<NewComment>,
) -> Result<HttpResponse> {
    let post_id = path.into_inner();
    let comment = Comment {
        id: Uuid::new_v4(),
        post_id,
        user_id: body.user_id,
        content: body.content.clone(),
        timestamp: Utc::now(),
        parent_id: body.parent_id,
    };

    let comment_id = comment_repo::create_comment(&pool, &comment)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(comment_id.to_string()))
}

#[get("/posts/{post_id}/comments")]
pub async fn get_comments(pool: Data<PgPool>, path: Path<Uuid>) -> Result<Json<Vec<Comment>>> {
    let post_id = path.into_inner();
    let comments = comment_repo::get_comments_by_post(&pool, post_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(Json(comments))
}

#[patch("/comments/{comments_id}")]
pub async fn update_comment(
    pool: Data<PgPool>,
    path: Path<Uuid>,
    body: String,
) -> Result<HttpResponse, actix_web::Error> {
    let comment_id = path.into_inner();
    let update_content = String::from(&body);

    let comment_id = comment_repo::update_comment(&pool, comment_id, update_content.clone())
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(format!("{} -> {}", comment_id.to_string(), update_content)))
}

#[delete("/comments/{comment_id}")]
pub async fn delete_comment(
    pool: Data<PgPool>,
    path: Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let comment_id = path.into_inner();

    comment_repo::delete_comment(&pool, comment_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(format!("{} was deleted", comment_id.to_string())))
}
