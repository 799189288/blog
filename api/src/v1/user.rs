use crate::auth::Claims;
use crate::auth::Keys;
use crate::response::CustomResponse;
use crate::response::Result;
use crate::validate::ValidatedForm;
use axum::extract::State;
use axum::routing::post;
use axum::Form;
use axum::Json;
use axum::Router;
use entity::user;
use jsonwebtoken::encode;
use jsonwebtoken::Header;
use service::sea_orm::prelude::Uuid;
use service::sea_orm::DatabaseConnection;

pub async fn new_user(
    State(db): State<DatabaseConnection>,
    ValidatedForm(form): ValidatedForm<user::InsertModel>,
) -> Result<Json<CustomResponse<Uuid>>> {
    let data = service::mutation::Mutation::create_user(&db, form.into()).await?;
    Ok(Json(CustomResponse::ok(data.id)))
}

pub async fn login_in(
    State(db): State<DatabaseConnection>,
    form: Form<user::LoginModel>,
) -> Result<Json<CustomResponse<String>>> {
    let form = form.0;
    let user = service::query::Query::check_user_exist(&db, form).await?;
    if let Some(user) = user {
        let claims = Claims {
            name: user.username,
            exp: chrono::Utc::now().timestamp() + 60 * 60 * 1000,
            user_id: user.id,
        };
        let token = encode(&Header::default(), &claims, &Keys::new(b"test").encoding)?;
        Ok(Json(CustomResponse::ok(token)))
    } else {
        Err(anyhow::anyhow!("未找到该用户").into())
    }
}

pub fn route() -> Router<DatabaseConnection> {
    let router = Router::new()
        .route("/user/new", post(new_user))
        .route("/user/login", post(login_in));
    router
}
