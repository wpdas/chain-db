use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::http::Method;
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
            kind: Kind::Response | Kind::Request,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        // Adicionar cabeçalhos CORS para todas as respostas
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, PUT, DELETE, OPTIONS",
        ));
        response.set_header(Header::new(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization, Accept, X-Requested-With, Origin",
        ));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        response.set_header(Header::new("Access-Control-Max-Age", "86400")); // 24 horas

        // Se for uma requisição OPTIONS (preflight), retornar 200 OK
        if request.method() == Method::Options {
            response.set_status(rocket::http::Status::Ok);
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut rocket::Data<'_>) {
        // Verificar se é uma requisição OPTIONS (preflight)
        if request.method() == Method::Options {
            // Não é necessário fazer nada aqui, apenas garantir que o método seja implementado
            // para que o Kind::Request seja processado
        }
    }
}

#[allow(dead_code)]
pub fn routes() -> Vec<rocket::Route> {
    routes![
        routes::cors::options_handler,
        routes::database::create_database,
        routes::database::connect_database,
        routes::database::change_password,
        routes::table::list_tables,
        routes::table::get_table_data,
        routes::table::get_document_by_id,
        routes::table::update_table,
        routes::table::persist_table,
        routes::table::get_history,
        routes::table::find_where,
        routes::table::find_where_advanced,
        routes::events::events_ws,
        routes::events::event_types
    ]
}
