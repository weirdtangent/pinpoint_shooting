//#![feature(proc_macro_hygiene, decl_macro)]

use rocket::get;
use rocket_contrib::templates::Template;
use serde_json::json;

#[get("/")]
pub fn index() -> rocket_contrib::templates::Template {
    let context = json!({"title": "Greeting", "greeting": "Welcome to templates"});
    Template::render("index", &context)
}
