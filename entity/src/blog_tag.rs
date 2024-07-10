//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "blog_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub blog_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::blog::Entity",
        from = "Column::BlogId",
        to = "super::blog::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Blog,
    #[sea_orm(
        belongs_to = "super::tag::Entity",
        from = "Column::TagId",
        to = "super::tag::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Tag,
}

impl Related<super::blog::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Blog.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
