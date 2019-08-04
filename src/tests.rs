use rocket::http::{ContentType, Status};
use rocket::local::Client;

use crate::rocket_prep;

#[test]
fn check_index() {
    let client = Client::new(rocket_prep()).unwrap();

    let response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::HTML));
}

#[test]
fn check_favicon() {
    let client = Client::new(rocket_prep()).unwrap();

    let response = client.get("/favicon.ico").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type(), Some(ContentType::Icon));
}
