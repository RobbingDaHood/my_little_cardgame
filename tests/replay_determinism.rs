// Randomized test to ensure GameState grant operations update balances correctly
use my_little_cardgame::library::types::token_balance_by_type;
use my_little_cardgame::library::{registry::TokenRegistry, types::TokenType, GameState};
use rand::{RngCore, SeedableRng};
use rand_pcg::Lcg64Xsh32;

#[test]
fn grant_operations_produce_correct_balances() {
    let mut rng = Lcg64Xsh32::from_seed([42u8; 16]);
    let token_ids: Vec<TokenType> = TokenRegistry::with_canonical()
        .tokens
        .keys()
        .cloned()
        .collect();

    for _ in 0..100 {
        let ops_count = (rng.next_u64() % 20 + 1) as usize;
        let mut gs = GameState::new();
        for _ in 0..ops_count {
            let idx = (rng.next_u64() as usize) % token_ids.len();
            let token_id = &token_ids[idx];
            let amount = (rng.next_u64() % 101) as i64; // only positive grants
            let before = token_balance_by_type(&gs.token_balances, token_id);
            if gs.apply_grant(token_id, amount, None).is_ok() {
                let after = token_balance_by_type(&gs.token_balances, token_id);
                assert_eq!(after, before + amount, "grant of {amount} to {token_id:?}");
            }
        }
    }
}
