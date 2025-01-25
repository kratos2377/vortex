pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240520_200614_add_table_in_friend_requests;
mod m20241119_200327_add_isonline_column;
mod m20241123_171604_game_bets;
mod m20250114_171855_create_session_in_game_bet;
mod m20250125_214938_game_bets_alter;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240520_200614_add_table_in_friend_requests::Migration),
            Box::new(m20241119_200327_add_isonline_column::Migration),
            Box::new(m20241123_171604_game_bets::Migration),
            Box::new(m20250114_171855_create_session_in_game_bet::Migration),
            Box::new(m20250125_214938_game_bets_alter::Migration),
        ]
    }
}
