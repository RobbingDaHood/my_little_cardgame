// Tests moved from src/library.rs
use my_little_cardgame::library::{
    action_log,
    types::{token_balance_by_type, ActionPayload, Token, TokenType},
    GameState,
};
use std::sync::Arc;
use std::thread;

#[test]
fn direct_balance_modifies_balance() {
    let mut gs = GameState::new();
    assert_eq!(
        token_balance_by_type(&gs.token_balances, &TokenType::Insight),
        0
    );
    gs.token_balances
        .insert(Token::persistent(TokenType::Insight), 10);
    assert_eq!(
        token_balance_by_type(&gs.token_balances, &TokenType::Insight),
        10
    );
}

#[test]
fn action_log_concurrent_append() {
    let log = Arc::new(action_log::ActionLog::new());
    let threads = 8usize;
    let per_thread = 100usize;
    let mut handles = Vec::new();
    for _i in 0..threads {
        let log_clone = Arc::clone(&log);
        handles.push(thread::spawn(move || {
            for j in 0..per_thread {
                let payload = ActionPayload::GrantToken {
                    token_id: TokenType::Insight,
                    amount: j as i64,
                    reason: None,
                    resulting_amount: j as i64,
                };
                log_clone.append("GrantToken", payload);
            }
        }));
    }
    for h in handles {
        h.join().expect("thread panicked");
    }
    let entries = log.entries();
    assert_eq!(entries.len(), threads * per_thread);
    let mut seqs: Vec<u64> = entries.iter().map(|e| e.seq).collect();
    seqs.sort();
    for (idx, seq) in seqs.iter().enumerate() {
        assert_eq!(*seq as usize, idx + 1);
    }
}
