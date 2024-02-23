use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        //todo!();

        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::First_Name).string().not_null())
                    .col(ColumnDef::new(User::Last_Name).string().not_null())
                    .col(ColumnDef::new(User::Password).string().not_null())
                    .col(ColumnDef::new(User::Email).string().not_null())
                    .col(ColumnDef::new(User::Username).string().not_null())
                    .col(ColumnDef::new(User::Verified).boolean().not_null().default(Value::from(false)))
                    .col(ColumnDef::new(User::Score).integer().not_null().default(Value::from(0)))
                    .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await;


            manager
            .create_table(
                Table::create()
                    .table(UsersFriends::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersFriends::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UsersFriends::UserId).uuid().not_null())
                    .col(ColumnDef::new(UsersFriends::FriendId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .name("fk-user_friend")
                        .from(UsersFriends::Table, UsersFriends::UserId)
                        .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await;

        
            manager
            .create_table(
                Table::create()
                    .table(UsersWallets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersWallets::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UsersWallets::UserId).uuid().not_null())
                    .col(ColumnDef::new(UsersWallets::WalletAddress).string().not_null())
                    .col(ColumnDef::new(UsersWallets::WalletType).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .name("fk-wallet_user_id")
                        .from(UsersWallets::Table, UsersWallets::UserId)
                        .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await;

            manager
            .create_table(
                Table::create()
                    .table(UsersFriendsRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersFriendsRequests::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UsersFriendsRequests::UserSentId).uuid().not_null())
                    .col(ColumnDef::new(UsersFriendsRequests::UserRecievedId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .name("fk-user_sent_id")
                        .from(UsersFriendsRequests::Table, UsersFriendsRequests::UserSentId)
                        .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await;


    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        todo!();

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await;


            manager
            .drop_table(Table::drop().table(UsersFriends::Table).to_owned())
            .await;

        manager
        .drop_table(Table::drop().table(UsersWallets::Table).to_owned())
        .await;

        manager
        .drop_table(Table::drop().table(UsersFriendsRequests::Table).to_owned())
        .await;
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    First_Name,
    Last_Name,
    Password,
    Email,
    Username,
    Verified,
    Score,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UsersFriends {
    Table,
    Id,
    UserId,
    FriendId,
}

#[derive(DeriveIden)]
enum UsersWallets {
    Table,
    Id,
    UserId,
    WalletAddress,
    WalletType,
}

#[derive(DeriveIden)]
enum UsersFriendsRequests {
    Table,
    Id,
    UserSentId,
    UserRecievedId,
}





