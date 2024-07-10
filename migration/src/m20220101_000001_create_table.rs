use extension::postgres::Type;
use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(StatusEnum)
                    .values(StatusVariants::iter())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .col(
                        ColumnDef::new(Category::Id)
                            .primary_key()
                            .uuid()
                            .extra("DEFAULT uuid_generate_v4()"),
                    )
                    .col(ColumnDef::new(Category::Name).string())
                    .col(ColumnDef::new(Category::CategoryId).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-category_id")
                            .from(Category::Table, Category::CategoryId)
                            .to(Category::Table, Category::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .primary_key()
                            .extra("DEFAULT uuid_generate_v4()"),
                    )
                    .col(ColumnDef::new(User::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(User::Password).string().not_null())
                    .col(ColumnDef::new(User::Email).string().not_null())
                    .col(ColumnDef::new(User::Avatar).text())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Blog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Blog::Id)
                            .uuid()
                            .primary_key()
                            .extra("DEFAULT uuid_generate_v4()"),
                    )
                    .col(ColumnDef::new(Blog::Title).string().not_null())
                    .col(ColumnDef::new(Blog::Content).string().not_null())
                    .col(ColumnDef::new(Blog::UserId).uuid().not_null())
                    .col(ColumnDef::new(Blog::CategoryId).uuid().not_null())
                    .col(
                        ColumnDef::new(Blog::CreateTime)
                            .timestamp()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(
                        ColumnDef::new(Blog::UpdateTime)
                            .timestamp()
                            .not_null()
                            .extra("DEFAULT NOW()"),
                    )
                    .col(ColumnDef::new(Blog::CoverImage).string())
                    .col(
                        ColumnDef::new(Blog::Status)
                            .enumeration(Alias::new("status_enum"), StatusVariants::iter())
                            .default(StatusVariants::Draft.to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog-user_id")
                            .from(Blog::Table, Blog::UserId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog-category_id")
                            .from(Blog::Table, Blog::CategoryId)
                            .to(Category::Table, Category::Id),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .col(
                        ColumnDef::new(Tag::Id)
                            .primary_key()
                            .uuid()
                            .extra("DEFAULT uuid_generate_v4()"),
                    )
                    .col(ColumnDef::new(Tag::Name).string())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BlogTag::Table)
                    .col(ColumnDef::new(BlogTag::BlogId).uuid().not_null())
                    .col(ColumnDef::new(BlogTag::TagId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .name("pk-blog_tag-tag_id")
                            .col(BlogTag::BlogId)
                            .col(BlogTag::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_tag-blog_id")
                            .from(BlogTag::Table, BlogTag::BlogId)
                            .to(Blog::Table, Blog::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_tag-tag_id")
                            .from(BlogTag::Table, BlogTag::TagId)
                            .to(Tag::Table, Tag::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BlogTag::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Blog::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Category::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(StatusEnum).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Category {
    Table,
    Id,
    Name,
    CategoryId,
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum BlogTag {
    Table,
    BlogId,
    TagId,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Username,
    Password,
    Email,
    Avatar,
}

#[derive(DeriveIden)]
enum Blog {
    Table,
    Id,
    Title,
    Content,
    UserId,
    CategoryId,
    CreateTime,
    UpdateTime,
    CoverImage,
    Status,
}

#[derive(DeriveIden)]
struct StatusEnum;

#[derive(DeriveIden, EnumIter)]
enum StatusVariants {
    Draft,
    Pend,
    Post,
}
