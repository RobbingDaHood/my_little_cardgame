// Tests moved from src/library.rs
use std::sync::Arc;
use std::thread;
use my_little_cardgame::library::{GameState, action_log, types::ActionPayload};

#[test]
fn grant_and_replay() {
    let mut gs = GameState::new();
    assert_eq!(gs.token_balances.get("Insight").copied().unwrap_or(0), 0);
    let entry = gs.apply_grant("Insight", 10).expect("apply_grant failed");
    assert_eq!(entry.seq, 1);
    assert_eq!(gs.token_balances.get("Insight").copied().unwrap_or(0), 10);

    // replay
    let replayed = GameState::replay_from_log(gs.registry.clone(), &gs.action_log);
    assert_eq!(replayed.token_balances.get("Insight").copied().unwrap_or(0), 10);
    assert_eq!(replayed.action_log.entries().len(), 1);
}

#[test]
fn action_log_concurrent_append() {
    let log = Arc::new(action_log::ActionLog::new());
    let threads = 8usize;
    let per_thread = 100usize;
    let mut handles = Vec::new();
    for i in 0..threads {
        let log_clone = Arc::clone(&log);
        handles.push(thread::spawn(move || {
            for j in 0..per_thread {
                let payload = ActionPayload::GrantToken { token_id: format!("t{}_{}", i, j), amount: j as i64 };
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
