use lazy_static::lazy_static;
use dotenv::dotenv;





lazy_static!{
    pub static ref JWT_SECRET_TOKEN: String = set_token();
}


pub const SMTP_HOST: &str = "smtp-relay.sendinblue.com";

fn set_token() -> String{
    dotenv().ok();
   // let token = env::var("JWT_SECRET_TOKEN").unwrap();
    "new-jwt-secret-token".to_string()

}