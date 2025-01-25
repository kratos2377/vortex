use axum::{http::request, Json};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Deserialize , Serialize )]
pub struct GetHealthNodeResponseJson {
   pub jsonrpc: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub id: u64
}



pub async fn get_solana_rpc_node_health(client: Client) -> bool {

let res = client.post("https://api.devnet.solana.com").header("Content-Type", "application/json").body(
    serde_json::to_string(&json!(
        {"jsonrpc":"2.0","id":1, "method":"getHealth"}
      )).unwrap()
).send().await;


if res.is_err() {
    return false
}

let res_body = res.unwrap().text().await.unwrap();


let response_json = serde_json::from_str(&res_body);

if response_json.is_err() {
    return false
}


let converted_model: GetHealthNodeResponseJson = response_json.unwrap();


if converted_model.error.is_some() {
    return false
}


return true
}