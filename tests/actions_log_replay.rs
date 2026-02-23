use my_little_cardgame::library::{registry::TokenRegistry, types::TokenType, GameState};

#[test]
fn replay_from_log_reproduces_state() {
    let mut gs = GameState::new();
    let _ = gs.apply_grant(&TokenType::Insight, 10, None).unwrap();
    let _ = gs.apply_grant(&TokenType::Renown, 5, None).unwrap();

    // clone action log and replay
    let log_clone = gs.action_log.clone();
    let registry = TokenRegistry::with_canonical();

    let replayed = GameState::replay_from_log(registry, &log_clone);
    assert_eq!(
        replayed.token_balances.get(&TokenType::Insight),
        Some(&10i64)
    );
    assert_eq!(replayed.token_balances.get(&TokenType::Renown), Some(&5i64));
}
