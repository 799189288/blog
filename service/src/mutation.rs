use ::entity::{blog, blog_tag, category, tag, user};
use sea_orm::*;
use uuid::Uuid;

pub struct Mutation {}

impl Mutation {
    pub async fn create_blog(
        db: &DbConn,
        form_data: blog::ActiveModel,
    ) -> Result<blog::ActiveModel, DbErr> {
        form_data.save(db).await
    }
    pub async fn create_user(
        db: &DbConn,
        form_data: user::ActiveModel,
    ) -> Result<user::Model, DbErr> {
        form_data.insert(db).await
    }
    pub async fn create_category(
        db: &DbConn,
        form_data: category::ActiveModel,
    ) -> Result<category::ActiveModel, DbErr> {
        form_data.save(db).await
    }
    pub async fn create_tag(db: &DbConn, form_data: tag::ActiveModel) -> Result<tag::Model, DbErr> {
        form_data.insert(db).await
    }
    pub async fn create_blog_tag(
        db: &DbConn,
        form_data: blog_tag::ActiveModel,
    ) -> Result<blog_tag::Model, DbErr> {
        form_data.insert(db).await
    }
    pub async fn delete_blog_tag(db: &DbConn, id: Uuid) -> Result<&str, DbErr> {
        let res = tag::Entity::find()
            .filter(tag::Column::Id.eq(id))
            .one(db)
            .await?;
        res.unwrap().delete(db).await?;
        Ok("删除成功")
    }
}
