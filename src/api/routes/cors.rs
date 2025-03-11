use rocket::http::Status;
use rocket::options;

// Rota para lidar com requisições OPTIONS para qualquer caminho
#[options("/<_..>")]
pub fn options_handler() -> Status {
    // O status 200 OK será retornado e os cabeçalhos CORS serão adicionados pelo fairing CORS
    Status::Ok
}
