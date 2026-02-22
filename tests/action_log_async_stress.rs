// Async stress test for ActionLog using tokio tasks
use my_little_cardgame::library::{
    action_log,
    types::{ActionPayload, TokenId},
};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn action_log_async_stress() {
    let log = Arc::new(action_log::ActionLog::new());
    let threads = 8usize;
    let per_thread = 100usize;
    let mut handles = Vec::new();
    for _i in 0..threads {
        let log_clone = Arc::clone(&log);
        handles.push(tokio::spawn(async move {
            for j in 0..per_thread {
                let payload = ActionPayload::GrantToken {
                    token_id: TokenId::Insight,
                    amount: j as i64,
                    reason: None,
                    resulting_amount: j as i64,
                };
                // append synchronously from async task (should be safe); yield occasionally
                log_clone.append("GrantToken", payload);
                if j % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }
        }));
    }
    for h in handles {
        h.await.expect("task panicked");
    }
    let entries = log.entries();
    assert_eq!(entries.len(), threads * per_thread);
    let mut seqs: Vec<u64> = entries.iter().map(|e| e.seq).collect();
    seqs.sort();
    for (idx, seq) in seqs.iter().enumerate() {
        assert_eq!(*seq as usize, idx + 1);
    }
}
