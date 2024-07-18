use std::iter::zip;

use ::entity::{
    blog::{self, CombineBlog},
    blog_tag, category, tag, user,
};
use sea_orm::*;
use uuid::Uuid;

pub struct Query {}

impl Query {
    pub async fn get_blog_list(db: &DbConn) -> Result<Vec<CombineBlog>, DbErr> {
        let blogs: Vec<blog::Model> = blog::Entity::find().all(db).await?;
        let categorys: Vec<Option<category::Model>> = blogs.load_one(category::Entity, db).await?;
        let tags: Vec<Vec<tag::Model>> = blogs
            .load_many_to_many(tag::Entity, blog_tag::Entity, db)
            .await?;
        let list = zip(blogs, categorys)
            .zip(tags)
            .map(|((a, b), c)| {
                let name;
                if let Some(b) = b {
                    name = b.name
                } else {
                    name = None
                }
                CombineBlog {
                    blog: a,
                    category: name,
                    tags: c.iter().filter_map(|i| i.clone().name).collect(),
                }
            })
            .collect();
        Ok(list)
    }
    pub async fn get_tag_list(db: &DbConn, id: Uuid) -> Result<Vec<tag::Model>, DbErr> {
        let tag_ids: Vec<Uuid> = blog_tag::Entity::find()
            .filter(blog_tag::Column::BlogId.eq(id))
            .all(db)
            .await?
            .into_iter()
            .map(|i| i.tag_id)
            .collect();
        let tags = tag::Entity::find()
            .filter(tag::Column::Id.is_in(tag_ids))
            .all(db)
            .await?;
        Ok(tags)
    }
    pub async fn check_user_exist(
        db: &DbConn,
        form: user::LoginModel,
    ) -> Result<Option<user::Model>, DbErr> {
        let user = user::Entity::find()
            .filter(user::Column::Username.eq(form.username))
            .filter(user::Column::Password.eq(form.password))
            .one(db)
            .await?;
        Ok(user)
    }
    pub async fn query_blog_category(
        db: &DbConn,
        id: Uuid,
    ) -> Result<Option<category::Model>, DbErr> {
        let category = category::Entity::find()
            .filter(category::Column::Id.eq(id))
            .one(db)
            .await?;
        Ok(category)
    }
    pub async fn query_category(db: &DbConn) -> Result<Vec<category::TreeModel>, DbErr> {
        let categories: Vec<category::TreeModel> = category::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|i| i.into())
            .collect();
        let first_categories = categories
            .clone()
            .into_iter()
            .filter(|i| i.category.category_id.is_none())
            .collect();
        let left_categories = categories
            .into_iter()
            .filter(|i| i.category.category_id.is_some())
            .collect();
        let categories = Self::recur_category(first_categories, left_categories);
        Ok(categories)
    }
    pub fn recur_category(
        mut categories: Vec<category::TreeModel>,
        left_categories: Vec<category::TreeModel>,
    ) -> Vec<category::TreeModel> {
        if categories.len() == 0 {
            return categories;
        }
        for node in categories.iter_mut() {
            let sub_categories: Vec<category::TreeModel> = left_categories
                .clone()
                .into_iter()
                .filter(|i| i.category.category_id.unwrap() == node.category.id)
                .collect();
            let children = Self::recur_category(sub_categories, left_categories.clone());
            node.children = children;
        }
        categories
    }
}
