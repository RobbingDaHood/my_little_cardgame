// Property-based tests to verify GameState replay over sequences and seq monotonicity
use my_little_cardgame::library::{registry::TokenRegistry, GameState};
use proptest::prelude::*;

proptest! {
    #[test]
    fn proptest_replay_sequence_preserves_balances(
        seq in prop::collection::vec(
            (
                prop::sample::select(TokenRegistry::with_canonical().tokens.keys().cloned().collect::<Vec<_>>()),
                -1000i64..1000i64
            ),
            0..20
        )
    ) {
        let mut gs = GameState::new();
        for (token_id, amount) in &seq {
            gs.apply_grant(token_id, *amount).expect("apply_grant failed");
        }
        let replayed = GameState::replay_from_log(gs.registry.clone(), &gs.action_log);
        prop_assert_eq!(gs.token_balances, replayed.token_balances);
    }

    #[test]
    fn proptest_seq_monotonicity(
        seq in prop::collection::vec(
            (
                prop::sample::select(TokenRegistry::with_canonical().tokens.keys().cloned().collect::<Vec<_>>()),
                -1000i64..1000i64
            ),
            0..20
        )
    ) {
        let mut gs = GameState::new();
        let mut seq_numbers: Vec<u64> = Vec::new();
        for (token_id, amount) in &seq {
            let entry = gs.apply_grant(token_id, *amount).expect("apply_grant failed");
            seq_numbers.push(entry.seq);
        }
        for (i, s) in seq_numbers.iter().enumerate() {
            prop_assert_eq!(*s as usize, i + 1);
        }
    }
}
