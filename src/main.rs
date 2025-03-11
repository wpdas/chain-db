use std::error::Error;

mod api;
mod chaindb;
mod config;
mod encryption;
mod errors;
mod events;
mod table;

#[cfg(test)]
mod tests;

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let figment = rocket::Config::figment()
        .merge(("port", 2818))
        .merge(("address", "0.0.0.0"))
        .merge(("log_level", rocket::config::LogLevel::Debug));

    let _rocket = rocket::custom(figment)
        .attach(api::CORS)
        .mount("/api/v1", api::routes())
        .launch()
        .await?;

    Ok(())
}
