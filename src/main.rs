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
        .mount("/post_contract_transaction", routes![routes::contract_transaction::post])
        // Returns the most recent transaction only under a specific contract
        .mount("/get_last_contract_transaction", routes![routes::get_contract_transaction::get])
        // Return a list of transactions under a specific contract
        .mount("/get_contract_transactions", routes![routes::get_contract_transactions::get])
        .mount("/chain", routes![routes::get_chain::get])
        .mount("/mine", routes![routes::mine_unconfirmed_transactions::get])

    
}