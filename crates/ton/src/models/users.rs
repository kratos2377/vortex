use sea_orm::{entity::prelude::*, DeleteMany};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

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
    pub first_name: String,
    pub last_name: String,
    pub verified: bool,
    pub score: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_online: bool,
}



#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::users_friends::Entity")]
    UsersFriends,

    #[sea_orm(has_many = "super::users_wallet_keys::Entity")]
    UsersWallets,

    #[sea_orm(has_many = "super::users_friends_requests::Entity")]
    UsersFriendsRequests,
}

impl Related<super::users_friends::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersFriends.def()
    }
}

impl Related<super::users_friends_requests::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersFriendsRequests.def()
    }
}

impl Related<super::users_wallet_keys::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersWallets.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_id(id: Uuid) -> Select<Entity> {
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