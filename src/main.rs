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

    // let key = "261a2abee90eb2e6dcbca892946613a8ab2d2674021a2c314df4abda92501a45";
    // let msg = "fala mo kirido meu bagulho doido, assim assim, do jeito que tem que ser";
    // let a1 = AesEcb::encode(msg, key);
    // println!("A1: {:?}", a1);
    // let a2 = AesEcb::decode(&a1, "261a2abee90eb2e6dcbca892946613a8ab2d2674021a2c314df4abda92501a45");

    // if a2.is_some() {
    //     println!("A2: {:?}", a2);
    //     println!("Eq: {:?}", a2.expect("Error decoding") == msg.to_string());
    // }

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
        .mount("/chain", routes![routes::get_chain::get])
        .mount("/mine", routes![routes::mine_unconfirmed_transactions::get])
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
