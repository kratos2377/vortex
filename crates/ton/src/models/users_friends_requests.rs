use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Debug, Deserialize , Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "users_friends_requests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_sent_id: Uuid,
    pub user_recieved_id: Uuid,
    pub user_sent_username: String,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserSentId",
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

    pub fn find_by_user_received_id(id: &Uuid) -> Select<Entity> {
        Self::find().filter(Column::UserRecievedId.eq(*id))
    }

    pub fn find_by_user_id_and_received_id(user_id: &Uuid, received_id: &Uuid) -> Select<Entity> {
        Self::find().filter(Column::UserSentId.eq(*user_id)).filter(Column::UserRecievedId.eq(*received_id))
    }
}