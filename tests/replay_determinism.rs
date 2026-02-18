// Randomized test to ensure GameState::replay_from_log reproduces token balances
use my_little_cardgame::library::{registry::TokenRegistry, GameState};
use rand::{RngCore, SeedableRng};
use rand_pcg::Lcg64Xsh32;

#[test]
fn replay_from_log_reproduces_balances_randomized() {
    let mut rng = Lcg64Xsh32::from_seed([42u8; 16]);
    let token_ids: Vec<String> = TokenRegistry::with_canonical()
        .tokens
        .keys()
        .cloned()
        .collect();

    // Run a number of randomized scenarios deterministically
    for _ in 0..100 {
        let ops_count = (rng.next_u64() % 20 + 1) as usize;
        let mut gs = GameState::new();
        for _ in 0..ops_count {
            let idx = (rng.next_u64() as usize) % token_ids.len();
            let token_id = token_ids[idx].clone();
            let amount = (rng.next_u64() % 2001) as i64 - 1000; // range -1000..1000
            gs.apply_grant(&token_id, amount)
                .expect("apply_grant failed");
        }
        let gs2 = GameState::replay_from_log(gs.registry.clone(), &gs.action_log);
        assert_eq!(gs.token_balances, gs2.token_balances);
    }
}
