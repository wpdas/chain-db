/**
 * Chain DB
 */
// Server - API
use rocket::http::Header;
use rocket::{Request, Response, Config, launch, routes};
use rocket::fairing::{Fairing, Info, Kind};

mod core_tables;
mod kibi;
mod routes;

// CORS
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
    // Set PORT
    let figment = Config::figment().merge(("port", 2818));

    rocket::custom(figment)
        .attach(CORS)
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
        // TODO: use env here
        // .mount("/chain", routes![routes::get_chain::get])
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
