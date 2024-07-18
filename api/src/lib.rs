use axum::{
    async_trait,
    extract::{rejection::FormRejection, FromRequest, FromRequestParts, Multipart, Request, State},
    http::{request::Parts, StatusCode},
    middleware::from_extractor,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, RequestPartsExt, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use dotenvy::dotenv;
use entity::{
    blog::{self, CombineBlog, InsertModel},
    blog_tag,
    category::{self},
    tag, user,
};
use error::CustomError;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use migration::{extension::postgres::Extension, ConnectionTrait, PostgresQueryBuilder};
use response::CustomResponse;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use service::sea_orm::{
    prelude::Uuid, ConnectOptions, Database, DatabaseBackend, DatabaseConnection, Statement,
    TryIntoModel,
};
use std::{env, time::Duration};
use util::stream_to_file;
use validator::Validate;
mod error;
mod response;
mod util;
pub type Result<T> = std::result::Result<T, CustomError>;
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap();
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let db = Database::connect(opt).await.unwrap();
    // 先创建uuid扩展
    let stmt = Extension::create()
        .name(r#""uuid-ossp""#)
        .if_not_exists()
        .to_string(PostgresQueryBuilder);
    db.execute(Statement::from_string(DatabaseBackend::Postgres, stmt))
        .await?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    let auth_route = Router::new()
        .route("/blog/new", post(new_blog))
        .route_layer(from_extractor::<Claims>());
    let app = Router::new()
        .route("/", get(hello_world))
        .nest("/", auth_route)
        .route("/get/blogs", get(get_blogs))
        .route("/user/new", post(new_user))
        .route("/user/login", post(login_in))
        .route("/tag/new", post(new_tag))
        .route("/category/new", post(new_category))
        .route("/category/list", get(get_categories))
        .route("/upload", post(upload_file))
        .with_state(db)
        .fallback(handle_rejection);
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
struct ValidatedForm<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = CustomError;

    async fn from_request(req: Request, state: &S) -> Result<Self> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_id: Uuid,
    name: String,
    exp: i64,
}
struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = String;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| "Failed to extract authorization header")?;
        // Decode the user data
        let token_data = decode::<Claims>(
            bearer.token(),
            &Keys::new(b"test").decoding,
            &Validation::default(),
        )
        .map_err(|err| err.to_string())?;

        Ok(token_data.claims)
    }
}

async fn handle_rejection() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}

async fn hello_world() -> impl IntoResponse {
    Html("hello_world")
}

async fn new_tag(
    State(db): State<DatabaseConnection>,
    Form(form): Form<tag::InsertModel>,
) -> Result<Json<CustomResponse<tag::Model>>> {
    let data = service::mutation::Mutation::create_tag(&db, form.into()).await?;
    Ok(Json(CustomResponse::ok(data)))
}

async fn new_blog(
    State(db): State<DatabaseConnection>,
    claims: Claims,
    Json(form): Json<blog::ReqModel>,
) -> Result<Json<CustomResponse<CombineBlog>>> {
    let insert_form = InsertModel {
        user_id: claims.user_id,
        id: form.id,
        title: form.title,
        content: form.content,
        category_id: form.category_id,
        cover_image: form.cover_image,
    };
    let data = service::mutation::Mutation::create_blog(&db, insert_form.into()).await?;
    let blog_model = data.try_into_model().unwrap();
    let blog_tag_list: Vec<Uuid> = service::query::Query::get_tag_list(&db, blog_model.id)
        .await?
        .into_iter()
        .map(|i| i.id)
        .collect();

    println!("{:?}", blog_tag_list);
    for blog_tag in blog_tag_list.iter() {
        if !form.tags.contains(blog_tag) {
            service::mutation::Mutation::delete_blog_tag(&db, *blog_tag).await?;
        }
    }
    for tag_id in form.tags.iter() {
        if !blog_tag_list.contains(tag_id) {
            let blog_tag = blog_tag::Model {
                blog_id: blog_model.id,
                tag_id: *tag_id,
            };
            service::mutation::Mutation::create_blog_tag(&db, blog_tag.into()).await?;
        };
    }
    let tags = service::query::Query::get_tag_list(&db, blog_model.id)
        .await?
        .into_iter()
        .map(|i| i.name.unwrap())
        .collect();
    let category = service::query::Query::query_blog_category(&db, blog_model.category_id).await?;
    Ok(Json(CustomResponse::ok(CombineBlog {
        blog: blog_model,
        tags,
        category: category.unwrap().name,
    })))
}

async fn new_user(
    State(db): State<DatabaseConnection>,
    ValidatedForm(form): ValidatedForm<user::InsertModel>,
) -> Result<Json<CustomResponse<Uuid>>> {
    let data = service::mutation::Mutation::create_user(&db, form.into()).await?;
    Ok(Json(CustomResponse::ok(data.id)))
}

async fn new_category(
    State(db): State<DatabaseConnection>,
    form: Form<category::InsertModel>,
) -> Result<Json<CustomResponse<category::Model>>> {
    let form = form.0;
    let data = service::mutation::Mutation::create_category(&db, form.into()).await?;
    Ok(Json(CustomResponse::ok(data.try_into_model().unwrap())))
}

async fn get_blogs(
    State(db): State<DatabaseConnection>,
) -> Result<Json<CustomResponse<Vec<CombineBlog>>>> {
    let list = service::query::Query::get_blog_list(&db).await?;
    Ok(Json(CustomResponse::ok(list)))
}

async fn login_in(
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

async fn get_categories(
    State(db): State<DatabaseConnection>,
) -> Result<Json<CustomResponse<Vec<category::TreeModel>>>> {
    let list = service::query::Query::query_category(&db).await?;
    Ok(Json(CustomResponse::ok(list)))
}

async fn upload_file(mut multipart: Multipart) -> Result<()> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.file_name().unwrap().to_string();
        stream_to_file(&name, field).await?;
    }
    Ok(())
}
