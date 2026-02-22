// Tests moved from src/library.rs
use my_little_cardgame::library::{
    action_log,
    types::{ActionPayload, TokenId},
    GameState,
};
use std::sync::Arc;
use std::thread;

#[test]
fn grant_and_replay() {
    let mut gs = GameState::new();
    assert_eq!(
        gs.token_balances
            .get(&TokenId::Insight)
            .copied()
            .unwrap_or(0),
        0
    );
    let entry = gs
        .apply_grant(&TokenId::Insight, 10, None)
        .expect("apply_grant failed");
    assert_eq!(entry.seq, 1);
    assert_eq!(
        gs.token_balances
            .get(&TokenId::Insight)
            .copied()
            .unwrap_or(0),
        10
    );

    // replay
    let replayed = GameState::replay_from_log(gs.registry.clone(), &gs.action_log);
    assert_eq!(
        replayed
            .token_balances
            .get(&TokenId::Insight)
            .copied()
            .unwrap_or(0),
        10
    );
    assert_eq!(replayed.action_log.entries().len(), 1);
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
                    token_id: TokenId::Insight,
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
