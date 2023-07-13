/**
 * Chain DB
 */

// Server - API
extern crate rocket;
use kibi::encryption::AesEcb;
use rocket::{routes, launch, Config};

mod routes;

// Blockchain - Kibi (yes this is my blockchain name)
mod kibi;

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
        .mount("/post_contract_transaction", routes![routes::contract_transaction::post])
        // Returns the most recent transaction only under a specific contract
        .mount("/get_last_contract_transaction", routes![routes::get_contract_transaction::get])
        // Return a list of transactions under a specific contract
        .mount("/get_contract_transactions", routes![routes::get_contract_transactions::get])
        .mount("/chain", routes![routes::get_chain::get])
        .mount("/mine", routes![routes::mine_unconfirmed_transactions::get])

    
}