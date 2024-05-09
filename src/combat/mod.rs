use rocket::response::status::Created;
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;
use rocket::yansi::Paint;
use rocket_okapi::{JsonSchema, openapi};

use crate::combat::units::{get_gnome, Unit};
use crate::deck::{Card, Deck};
use crate::deck::card::CardCreate;
use crate::player_data::PLayerData;

pub mod units;


#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
pub struct Combat {
    pub allies: Vec<Unit>,
    pub enemies: Vec<Unit>,
}

#[openapi]
#[get("/combat")]
pub async fn get_combat(player_data: &State<PLayerData>) -> Json<Option<Combat>> {
    Json(*player_data.current_combat.lock().await.clone())
}

#[openapi]
#[post("/combat")]
pub async fn initialize_combat(player_data: &State<PLayerData>) -> Created<&str> {
    player_data.current_combat.lock().await.replace(Combat {
        allies: vec![],
        enemies: vec![get_gnome()],
    });
    Created::new("/combat")
}
