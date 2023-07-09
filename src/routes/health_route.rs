use rocket::get;

#[get("/")]
pub fn get() -> &'static str {
  "Health Ok"
}