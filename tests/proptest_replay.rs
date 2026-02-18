// Property-based test to verify GameState replay preserves token balances
use my_little_cardgame::library::{registry::TokenRegistry, GameState};
use proptest::prelude::*;

proptest! {
    #[test]
    fn proptest_replay_preserves_balances(
        token_id in prop::sample::select(TokenRegistry::with_canonical().tokens.keys().cloned().collect::<Vec<_>>()),
        amount in -1000i64..1000i64
    ) {
        let mut gs = GameState::new();
        let _prev = gs.token_balances.get(&token_id).copied().unwrap_or(0);
        gs.apply_grant(&token_id, amount).expect("apply_grant failed");
        let replayed = GameState::replay_from_log(gs.registry.clone(), &gs.action_log);
        prop_assert_eq!(gs.token_balances, replayed.token_balances);
    }
}
