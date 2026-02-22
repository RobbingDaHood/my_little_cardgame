use std::sync::Arc;

use rand::{RngCore, SeedableRng};
use rand_pcg::Lcg64Xsh32;
use rocket::futures::lock::Mutex;

use crate::area_deck::AreaDeck;

pub struct PlayerData {
    #[allow(dead_code)]
    pub(crate) seed: Arc<Mutex<[u8; 16]>>,
    #[allow(dead_code)]
    pub(crate) random_generator_state: Arc<Mutex<Lcg64Xsh32>>,
    /// Represents the player's current location/area
    pub(crate) current_area_deck: Arc<Mutex<Option<AreaDeck>>>,
}

pub fn new() -> PlayerData {
    let mut new_seed: [u8; 16] = [1; 16];
    Lcg64Xsh32::from_entropy().fill_bytes(&mut new_seed);
    let random_generator = Lcg64Xsh32::from_seed(new_seed);

    PlayerData {
        seed: Arc::new(Mutex::new(new_seed)),
        random_generator_state: Arc::new(Mutex::new(random_generator)),
        current_area_deck: Arc::new(Mutex::new(Some(initialize_area_deck()))),
    }
}

fn initialize_area_deck() -> AreaDeck {
    let mut deck = AreaDeck::new("starter_area".to_string());
    // Reference the gnome CombatEncounter card at Library index 3
    deck.add_encounter(3);
    deck.add_encounter(3);
    deck.add_encounter(3);
    deck
}
