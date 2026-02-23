// Property-based test to verify GameState grant operations
use my_little_cardgame::library::types::token_balance_by_type;
use my_little_cardgame::library::{registry::TokenRegistry, GameState};
use proptest::prelude::*;

proptest! {
    #[test]
    fn proptest_grant_updates_balance(
        token_id in prop::sample::select(TokenRegistry::with_canonical().tokens.keys().cloned().collect::<Vec<_>>()),
        amount in -1000i64..1000i64
    ) {
        let mut gs = GameState::new();
        let prev = token_balance_by_type(&gs.token_balances, &token_id);
        gs.apply_grant(&token_id, amount, None).expect("apply_grant failed");
        let after = token_balance_by_type(&gs.token_balances, &token_id);
        prop_assert_eq!(after, prev + amount);
    }
}
