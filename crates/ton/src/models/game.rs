use sea_orm::{entity::prelude::*, Condition, DeleteMany};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize , Serialize, DeriveEntityModel)]
#[sea_orm(table_name = "game")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub game_id: Uuid,
    pub session_id: String,
    pub is_stake_allowed: bool,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}



impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub fn find_by_id(id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::Id.eq(id))
    }


    pub fn delete_by_id(id: i32) -> DeleteMany<Entity> {
        Self::delete_many().filter(Column::Id.eq(id))
    }

    pub fn find_by_game_id(game_id: Uuid) -> Select<Entity> {
        Self::find().filter(Column::GameId.eq(game_id))
    }





    


    pub fn find_by_game_id_and_session_id(game_id: Uuid ,  session_id: String) -> Select<Entity> {
        Self::find().filter(
            Condition::all()
            .add(Column::GameId.eq(game_id))
            .add(Column::SessionId.eq(session_id))
        )
    }


    
}