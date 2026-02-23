use my_little_cardgame::library::types::{token_balance_by_type, TokenType};
use my_little_cardgame::library::GameState;

#[test]
fn grant_operations_update_balances() {
    let mut gs = GameState::new();
    gs.apply_grant(&TokenType::Insight, 10, None).unwrap();
    gs.apply_grant(&TokenType::Renown, 5, None).unwrap();

    assert_eq!(
        token_balance_by_type(&gs.token_balances, &TokenType::Insight),
        10
    );
    assert_eq!(
        token_balance_by_type(&gs.token_balances, &TokenType::Renown),
        5
    );
}
