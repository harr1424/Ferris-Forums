use crate::model::user::{DbAddUser, NewUser, User};
use crate::repo::user as user_repo;
use actix_web::{delete, get, patch, post, web::Data, web::Json, web::Path, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;

#[post("/users")]
pub async fn create_user(
    pool: Data<PgPool>,
    body: Json<NewUser>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let hashed_password = User::hash_password(&body.password)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    let user = DbAddUser {
        username: body.username.clone(),
        password_hash: hashed_password,
        is_moderator: body.is_moderator,
        created_at: Utc::now(),
    };

    let user_id = user_repo::create_user(&pool, &user)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(user_id.to_string()))
}

#[get("/users/{user_id}")]
pub async fn get_user_by_id(
    pool: Data<PgPool>,
    path: Path<i32>,
) -> Result<Json<User>, actix_web::Error> {
    let user_id = path.into_inner();

    let user = user_repo::get_user_by_id(&pool, user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(Json(user))
}

#[get("/users/{sub_name}")]
pub async fn get_user_by_username(
    pool: Data<PgPool>,
    path: Path<String>,
) -> Result<Json<User>, actix_web::Error> {
    let username = path.into_inner();

    let user = user_repo::get_user_by_username(&pool, &username)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(Json(user))
}

#[get("/users/{sub_name}")]
pub async fn get_users_by_sub(
    pool: Data<PgPool>,
    path: Path<String>,
) -> Result<Json<Vec<User>>, actix_web::Error> {
    let sub_name = path.into_inner();

    let users = user_repo::get_users_by_sub(&pool, &sub_name)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(Json(users))
}

#[get("/users/auth/{user_id}")]
pub async fn verify_user_password(
    pool: Data<PgPool>,
    path: Path<i32>,
    body: String,
) -> Result<Json<bool>, actix_web::Error> {
    let user_id = path.into_inner();
    let password_attempt = body;

    let user = user_repo::get_user_by_id(&pool, user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let verified = user.verify_password(&password_attempt);

    match verified {
        Ok(true) => Ok(Json(true)),
        Ok(false) => Ok(Json(false)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/users/exists/{username}")]
pub async fn username_exists(
    pool: Data<PgPool>,
    path: Path<String>,
) -> Result<Json<bool>, actix_web::Error> {
    let username = path.into_inner();

    match user_repo::username_exists(&pool, &username).await {
        Ok(Some(_user)) => Ok(Json(true)),
        Ok(None) => Ok(Json(false)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[patch("/users/mods/add/{user_id}")]
pub async fn grant_mod_status(
    pool: Data<PgPool>,
    path: Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();

    let user_id = user_repo::grant_mod_status(&pool, user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(format!("{} is now a moderator", user_id.to_string())))
}

#[patch("/users/mods/remove/{user_id}")]
pub async fn remove_mod_status(
    pool: Data<PgPool>,
    path: Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();

    let user_id = user_repo::remove_mod_status(&pool, user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(format!("{} is no longer a moderator", user_id.to_string())))
}

#[patch("/users/creds/{user_id}")]
pub async fn update_user_password(
    pool: Data<PgPool>,
    path: Path<i32>,
    body: String,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();
    let new_password_hash = User::hash_password(&body)
        .map_err(|e| Box::<dyn std::error::Error>::from(e.to_string()))?;

    let user_id = user_repo::update_user_password(&pool, user_id, &new_password_hash)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(format!("{} password has been updated", user_id.to_string())))
}

#[delete("/users/{user_id}")]
pub async fn delete_user(
    pool: Data<PgPool>,
    path: Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = path.into_inner();

    let user_id = user_repo::delete_user(&pool, user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().body(format!("{} has been deleted", user_id.to_string())))
}

#[cfg(test)]
mod user_api_tests {
    use super::*;
    use actix_web::{test, App};
    use sqlx::postgres::PgPoolOptions;
    use sqlx::PgPool;
    use std::env;

    async fn setup_db() -> PgPool {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in .env or environment variables");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Could not connect to the database");

        pool
    }

    #[actix_rt::test]
    async fn test_create_user() {
        let pool = setup_db();

        let app =
            test::init_service(App::new().app_data(Data::new(pool)).service(create_user)).await;

        let new_user = NewUser {
            username: "testuser".to_string(),
            password: "password123".to_string(),
            is_moderator: false,
        };

        let req = test::TestRequest::post()
            .uri("/users")
            .set_json(&new_user)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_get_user_by_id() {
        let pool = setup_db();

        let app =
            test::init_service(App::new().app_data(Data::new(pool)).service(get_user_by_id)).await;

        let user_id = 1;
        let req = test::TestRequest::get()
            .uri(&format!("/users/{}", user_id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
