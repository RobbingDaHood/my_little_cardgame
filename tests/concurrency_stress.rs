// Stress test for ActionLog concurrent append
use my_little_cardgame::library::{action_log, types::ActionPayload};
use std::sync::Arc;
use std::thread;

#[test]
fn action_log_stress_append() {
    let log = Arc::new(action_log::ActionLog::new());
    let threads = 16usize;
    let per_thread = 1000usize;
    let mut handles = Vec::new();
    for i in 0..threads {
        let log_clone = Arc::clone(&log);
        handles.push(thread::spawn(move || {
            for j in 0..per_thread {
                let payload = ActionPayload::GrantToken {
                    token_id: format!("t{}_{}", i, j),
                    amount: j as i64,
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
