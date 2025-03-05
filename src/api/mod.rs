use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
// use rocket::http::Method;
use rocket::routes;
use rocket::{Request, Response};
// use rocket_cors::{AllowedHeaders, AllowedOrigins};

pub mod auth;
pub mod models;
mod routes;

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, DELETE, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[allow(dead_code)]
pub fn routes() -> Vec<rocket::Route> {
    routes![
        routes::database::create_database,
        routes::database::connect_database,
        routes::database::change_password,
        routes::table::get_table_data,
        routes::table::update_table,
        routes::table::persist_table,
        routes::table::get_history,
        routes::table::find_where,
        routes::table::find_where_advanced,
        routes::events::events_ws,
        routes::events::event_types
    ]
}
