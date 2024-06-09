use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize , Serialize , Clone)]
pub struct ChessNormalEvent  {
    pub initial_cell: String,
    pub target_cell: String,
    pub piece: String,
}


#[derive(Debug, Deserialize , Serialize , Clone)]
pub struct ChessPromotionEvent  {
    pub initial_cell: String,
    pub target_cell: String,
    pub promoted_to: String,
    pub piece: String
}