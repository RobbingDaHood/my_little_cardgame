use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket_okapi::JsonSchema;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Status {
    message: String,
}

pub fn new_status(message: String) -> Json<Status> {
    Json(Status { message })
}
