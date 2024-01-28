use sea_orm::{entity::prelude::*, DeleteMany};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Debug, Deserialize , Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub password: String,
    #[sea_orm(unique)]
    pub username: String,
    #[sea_orm(unique)]
    pub email: String,
    first_name: String,
    last_name: String,
    score: u64,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at:  Option<chrono::DateTime<chrono::Utc>>,
    #[sea_orm(column_type = "Text", nullable)]
    pub token: Option<String>,
}



#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::users_friends::Entity")]
    UsersFriends,
}

impl Related<super::users_friends::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersFriends.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_id(id: i32) -> Select<Entity> {
        Self::find().filter(Column::Id.eq(id))
    }

    pub fn find_by_email(email: &str) -> Select<Entity> {
        Self::find().filter(Column::Email.eq(email))
    }

    pub fn find_by_username(username: &str) -> Select<Entity> {
        Self::find().filter(Column::Username.eq(username))
    }

    pub fn delete_by_id(id: i32) -> DeleteMany<Entity> {
        Self::delete_many().filter(Column::Id.eq(id))
    }
}