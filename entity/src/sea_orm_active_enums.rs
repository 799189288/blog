//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.12

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "status_enum")]
pub enum StatusEnum {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "pend")]
    Pend,
    #[sea_orm(string_value = "post")]
    Post,
}
