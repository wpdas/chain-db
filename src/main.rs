/**
 * Chain DB
 */
// Server - API
extern crate rocket;
use rocket::{launch, routes, Config};

// Blockchain - Kibi (yes this is my blockchain name)
mod core_tables;
mod kibi;
mod routes;

#[launch]
fn rocket() -> _ {
    // Set PORT
    let figment = Config::figment().merge(("port", 2818));

    rocket::custom(figment)
        .mount("/", routes![routes::health_route::get])
        .mount(
            "/post_contract_transaction",
            routes![routes::contract_transaction::post],
        )
        // Returns the most recent transaction only under a specific contract
        .mount(
            "/get_last_contract_transaction",
            routes![routes::get_contract_transaction::get],
        )
        // Return a list of transactions under a specific contract
        .mount(
            "/get_contract_transactions",
            routes![routes::get_contract_transactions::get],
        )
        // FOR DEBUG PURPOSES ONLY
        .mount("/chain", routes![routes::get_chain::get])
        // Create user account
        .mount(
            "/create_user_account",
            routes![routes::create_user_account::post],
        )
        // Get user account (login check)
        .mount("/get_user_account", routes![routes::get_user_account::get])
        // Get user account by id
        .mount(
            "/get_user_account_by_id",
            routes![routes::get_user_account_by_id::get],
        )
        // Transfer units between users
        .mount("/transfer_units", routes![routes::transfer_units::post])
        // Get Transfer Record by user id
        .mount(
            "/get_transfer_by_user_id",
            routes![routes::get_transfer_by_user_id::get],
        )
        // Get all Transfer Records by user id
        .mount(
            "/get_all_transfers_by_user_id",
            routes![routes::get_all_transfers_by_user_id::get],
        )
}
