use sea_orm::{entity::prelude::*, Condition, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::users;


#[derive(Clone, Debug, Serialize , Deserialize , DeriveEntityModel)]
#[sea_orm(table_name = "users_friends")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub friend_id: Uuid,
}   

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
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
    pub fn find_by_id(id: &Uuid) -> Select<Entity> {
        Self::find().filter(Column::Id.eq(*id))
    }

    pub fn find_by_user_and_friend_id(user_id: &Uuid, friend_id: &Uuid) -> Select<Entity> {
        Self::find().filter(Column::UserId.eq(*user_id)).filter(Column::FriendId.eq(*friend_id))
    }

    pub fn find_by_user_id(user_id: &Uuid) -> Select<Entity> {
        Self::find().filter(Column::UserId.eq(*user_id))
    }

    // pub fn find_user_online_friends(user_id: &Uuid) -> Select<Entity> {
    //     Self::find().join(sea_orm::JoinType::LeftJoin , Self::to()).filter(
    //         Condition::all().add(Column::UserId.eq(*user_id)).add(users::Column::IsOnline.eq(true))
    //     )
    // }

}