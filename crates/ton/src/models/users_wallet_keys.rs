use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Deserialize , Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "users_wallets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: String,
    #[sea_orm(unique)]
    pub wallet_address: String,
    pub wallet_type: String,
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

    pub fn find_by_userid(userid: &str) -> Select<Entity> {
        Self::find().filter(Column::UserId.eq(userid))
    }

    pub fn find_by_wallettype(wallettype: &str) -> Select<Entity> {
        Self::find().filter(Column::WalletType.eq(wallettype))
    }

    pub fn find_by_wallet_address(wallet_address: &str) -> Select<Entity> {
        Self::find().filter(Column::WalletAddress.eq(wallet_address))
    }


}