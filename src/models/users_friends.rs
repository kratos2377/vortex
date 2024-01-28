use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Clone, Debug, Serialize , Deserialize , DeriveEntityModel)]
#[sea_orm(table_name = "users_friends")]
pub struct Model {
    #[sea_orm(primary_key)]
    id: Uuid,
    userid: Uuid,
    friendid: Uuid,
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