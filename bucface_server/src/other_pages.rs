use actix_web::get;

#[get("/")]
pub async fn home() -> String {
    "Hello, world!".to_string()
}
