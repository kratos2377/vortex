use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize , Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "users_wallets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub userid: String,
    #[sea_orm(unique)]
    pub walletaddress: String,
    pub wallettype: String,
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

    pub fn find_by_userid(userid: &str) -> Select<Entity> {
        Self::find().filter(Column::Userid.eq(userid))
    }

    pub fn find_by_wallettype(wallettype: &str) -> Select<Entity> {
        Self::find().filter(Column::Wallettype.eq(wallettype))
    }

}