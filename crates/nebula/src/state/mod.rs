use sea_orm::DatabaseConnection;


#[derive(Clone)]
pub struct AppDBState {
    pub conn: DatabaseConnection,
}
