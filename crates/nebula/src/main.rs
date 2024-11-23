use axum::Router;
use routes::user_game_bet_routes;
use sea_orm::Database;
use state::AppDBState;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;


pub mod conf;
pub mod controllers;
pub mod routes;
pub mod utils;
pub mod state;
pub mod errors;


#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>  {

    let config = conf::configuration::Configuration::load().unwrap();
    //  dotenv().ok();
  
      //Connect with database
      let connection = match Database::connect(config.postgres_url.url).await {
          Ok(connection) => connection,
          Err(e) => panic!("{:?}",e)
      };
  
      let state = AppDBState {conn: connection };

      let user_game_bet_routes = routes::user_game_bet_routes::create_user_game_bet_routes() ;
      let routes_all = Router::new()
                              .nest( "/api/v1/game_bets", user_game_bet_routes)
                              .layer(ServiceBuilder::new()
                                      .layer(CookieManagerLayer::new())
                                      .layer(CorsLayer::permissive()))
                              .with_state(state);
  
  
      // run it
      let listener = tokio::net::TcpListener::bind("127.0.0.1:3020")
          .await
          .unwrap();
      println!("listening on {}", listener.local_addr().unwrap());
      axum::serve(listener, routes_all).await.unwrap();
  
      Ok(())

}
