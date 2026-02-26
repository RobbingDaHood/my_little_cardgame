use my_little_cardgame::library::types::{token_balance_by_type, Token, TokenType};
use my_little_cardgame::library::GameState;

#[test]
fn direct_balance_updates() {
    let mut gs = GameState::new();
    gs.token_balances
        .insert(Token::persistent(TokenType::Insight), 10);
    gs.token_balances
        .insert(Token::persistent(TokenType::Renown), 5);

    assert_eq!(
        token_balance_by_type(&gs.token_balances, &TokenType::Insight),
        10
    );
    assert_eq!(
        token_balance_by_type(&gs.token_balances, &TokenType::Renown),
        5
    );
}
