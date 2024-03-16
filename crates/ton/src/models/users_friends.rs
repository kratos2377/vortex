use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Debug, Serialize , Deserialize , DeriveEntityModel)]
#[sea_orm(table_name = "users_friends")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub userid: Uuid,
    pub friendid: Uuid,
}   

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::Userid",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Users,
}

// `Related` trait has to be implemented by hand
impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_id(id: &str) -> Select<Entity> {
        Self::find().filter(Column::Id.eq(id))
    }

    pub fn find_by_user_and_friend_id(user_id: &str, friend_id: &str) -> Select<Entity> {
        Self::find().filter(Column::Userid.eq(user_id)).filter(Column::Friendid.eq(friend_id))
    }

    pub fn find_by_user_id(user_id: &str) -> Select<Entity> {
        Self::find().filter(Column::Userid.eq(user_id))
    }

}