use core::f64;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;



#[derive( Deserialize , Clone)]
pub struct Game {
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub id: Uuid,
    pub name: String,
    pub game_type: String,
    pub is_staked: bool,
    pub current_state: String,
    pub state_index: i64,
    pub description: String,
    pub staked_money_state: Option<StakedUsers>,
    pub poker_state: Option<PokerState>, 
}


#[derive(Deserialize , Clone)]
pub struct PokerState { 
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub id: Uuid,
    pub game_id: Uuid,
    pub pot_size: f64,
    pub current_turn: String,
    pub user_states: HashMap<String , Option<Poker>>
}


#[derive(Serialize , Deserialize , Clone)]

pub struct Poker {
    pub money_left: f64,
    pub go_all_in: bool,
    pub turns_left: i64,
}

#[derive(Serialize , Deserialize , Clone)]
pub struct StakedUsers {
    #[serde(with = "bson::serde_helpers::uuid_1_as_binary")]
    pub game_id: Uuid,
    pub money_staked: HashMap<String , f64>
}


