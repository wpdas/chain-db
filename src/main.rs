/**
 * Chain DB
 */

// Server - API
extern crate rocket;
use rocket::{routes, launch, Config};

mod routes;

// Blockchain - Kibi (yes this is my blockchain name)
mod kibi;

#[launch]
fn rocket() -> _ {
    // Set PORT
    let figment = Config::figment().merge(("port", 2818));

    rocket::custom(figment)
        .mount("/", routes![routes::health_route::get])
        .mount("/contract_transaction", routes![routes::contract_transaction::post])
        .mount("/contract_payload", routes![routes::get_contract_payload::get])
        .mount("/contract_payload_json", routes![routes::get_contract_payload_json::get])
        .mount("/chain", routes![routes::get_chain::get])
        .mount("/mine", routes![routes::mine_unconfirmed_transactions::get])

    
}