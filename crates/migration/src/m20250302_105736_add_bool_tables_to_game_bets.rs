use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
    

        manager
        .alter_table(
            Table::alter()
                .table(GameBets::Table)
                .add_column(
                    ColumnDef::new(GameBets::Is_Game_Valid)
                        .boolean(),
                )
                .add_column(
                    ColumnDef::new(GameBets::Won_Status)
                    .boolean(),
                )
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
   
        manager
            .drop_table(Table::drop().table(GameBets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GameBets {
    Table,
    Is_Game_Valid,
    Won_Status
}