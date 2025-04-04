use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {


        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::Is_Online)
                            .boolean()
                            .not_null().default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
        .alter_table(Table::alter().table(Users::Table).drop_column( Users::Is_Online).to_owned())
        .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Is_Online
}
