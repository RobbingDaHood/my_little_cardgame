#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    my_little_cardgame::rocket_initialize()
}