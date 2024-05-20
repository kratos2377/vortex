use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {


        manager
        .alter_table(
            Table::alter()
            .table(UsersFriendsRequests::Table)
            .add_column(    ColumnDef::new(UsersFriendsRequests::UserSentUsername)
            .text()
            .not_null(),
        )
            .to_owned(),
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {


        manager
            .alter_table(Table::alter().table(UsersFriendsRequests::Table).drop_column( UsersFriendsRequests::UserSentUsername).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UsersFriendsRequests {
    Table,
    UserSentUsername
}

