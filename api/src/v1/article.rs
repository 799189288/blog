use crate::auth::Claims;
use crate::response::{CustomResponse, Result};
use axum::middleware::from_extractor;
use axum::routing::{get, post};
use axum::Router;
use axum::{extract::State, Form, Json};
use entity::{blog, blog_tag, tag};
use entity::{blog::CombineBlog, category};
use service::sea_orm::{prelude::Uuid, DatabaseConnection, TryIntoModel};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_blogs, new_blog, new_category, get_categories, new_tag),
    components(schemas(CombineBlog, blog::Model))
)]
pub(crate) struct ArticleApi;

#[utoipa::path(
    get,
    path = "/get/blogs",
    responses(
        (status = 200, description = "List all blogs successfully", body = [CombineBlog] )
    )
)]
pub async fn get_blogs(
    State(db): State<DatabaseConnection>,
) -> Result<Json<CustomResponse<Vec<CombineBlog>>>> {
    let list = service::query::Query::get_blog_list(&db).await?;
    Ok(Json(CustomResponse::ok(list)))
}

#[utoipa::path(
    post,
    path = "/blog/new",
    responses(
        (status = 200, description = "New Blog", body = [CombineBlog])
    )
)]
pub async fn new_blog(
    State(db): State<DatabaseConnection>,
    claims: Claims,
    Json(form): Json<blog::ReqModel>,
) -> Result<Json<CustomResponse<CombineBlog>>> {
    let insert_form = blog::InsertModel {
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

#[utoipa::path(
    get,
    path = "/category/new",
    responses(
        (status = 200, description = "Create A Category", body = Category::Model)
    )
)]
pub async fn new_category(
    State(db): State<DatabaseConnection>,
    form: Form<category::InsertModel>,
) -> Result<Json<CustomResponse<category::Model>>> {
    let form = form.0;
    let data = service::mutation::Mutation::create_category(&db, form.into()).await?;
    Ok(Json(CustomResponse::ok(data.try_into_model().unwrap())))
}

#[utoipa::path(
    get,
    path = "/category/list",
    responses(
        (status = 200, description = "List all Categories")
    )
)]
pub async fn get_categories(
    State(db): State<DatabaseConnection>,
) -> Result<Json<CustomResponse<Vec<category::TreeModel>>>> {
    let list = service::query::Query::query_category(&db).await?;
    Ok(Json(CustomResponse::ok(list)))
}

#[utoipa::path(
    post,
    path = "/tag/new",
    responses(
        (status = 200, description = "New Tag", body = tag::Model)
    )
)]
pub async fn new_tag(
    State(db): State<DatabaseConnection>,
    Form(form): Form<tag::InsertModel>,
) -> Result<Json<CustomResponse<tag::Model>>> {
    let data = service::mutation::Mutation::create_tag(&db, form.into()).await?;
    Ok(Json(CustomResponse::ok(data)))
}

pub fn route() -> Router<DatabaseConnection> {
    let auth_route = Router::new()
        .route("/blog/new", post(new_blog))
        .route_layer(from_extractor::<Claims>());
    let router = Router::new()
        .merge(auth_route)
        .route("/get/blogs", get(get_blogs))
        .route("/tag/new", post(new_tag))
        .route("/category/new", post(new_category))
        .route("/category/list", get(get_categories));
    router
}
