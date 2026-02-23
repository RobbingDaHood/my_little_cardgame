use std::sync::Arc;

use rand::{RngCore, SeedableRng};
use rand_pcg::Lcg64Xsh32;
use rocket::futures::lock::Mutex;

pub struct PlayerData {
    #[allow(dead_code)]
    pub(crate) seed: Arc<Mutex<[u8; 16]>>,
    #[allow(dead_code)]
    pub(crate) random_generator_state: Arc<Mutex<Lcg64Xsh32>>,
}

pub fn new() -> PlayerData {
    let mut new_seed: [u8; 16] = [1; 16];
    Lcg64Xsh32::from_entropy().fill_bytes(&mut new_seed);
    let random_generator = Lcg64Xsh32::from_seed(new_seed);

    PlayerData {
        seed: Arc::new(Mutex::new(new_seed)),
        random_generator_state: Arc::new(Mutex::new(random_generator)),
    }
}
