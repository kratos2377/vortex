use sea_orm::{entity::prelude::*, Condition, DeleteMany};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize , Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "game_bets")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub game_id: Uuid,
    pub user_id_betting_on: Uuid,
    pub session_id: String,
    pub game_name: String,
    pub bet_amount: f64,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
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


impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_id(id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::Id.eq(id))
    }


    pub fn delete_by_id(id: i32) -> DeleteMany<Entity> {
        Self::delete_many().filter(Column::Id.eq(id))
    }

    pub fn find_by_user_id(user_id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::UserId.eq(user_id))
    }

    pub fn find_by_game_id(game_id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::GameId.eq(game_id))
    }

    pub fn find_by_user_id_and_game_name(game_name: String ,  user_id: Uuid) -> Select<Entity> {
        Self::find().filter(
            Condition::all()
            .add(Column::UserId.eq(user_id))
            .add(Column::GameName.eq(game_name))
        )
    }


    pub fn find_by_user_id_and_game_id(game_id: Uuid ,  user_id: Uuid) -> Select<Entity> {
        Self::find().filter(
            Condition::all()
            .add(Column::UserId.eq(user_id))
            .add(Column::GameId.eq(game_id))
        )
    }
    
}