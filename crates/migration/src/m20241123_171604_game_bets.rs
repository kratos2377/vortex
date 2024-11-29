use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(GameBets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GameBets::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GameBets::UserId).uuid().not_null())
                    .col(ColumnDef::new(GameBets::GameId).uuid().not_null())
                    .col(ColumnDef::new(GameBets::GameName).string().not_null())
                    .col(ColumnDef::new(GameBets::Status).string().not_null())
                    .col(ColumnDef::new(GameBets::BetAmount).double().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .name("fk-game-bets-user-id")
                        .from(GameBets::Table, GameBets::UserId)
                        .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(GameBets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GameBets {
    Table,
    Id,
    UserId,
    GameId,
    GameName,
    BetAmount,
    Status,
}


#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}