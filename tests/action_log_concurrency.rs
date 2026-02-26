use std::sync::Arc;
use std::thread;

use my_little_cardgame::library::{action_log::ActionLog, types::ActionPayload};

#[test]
fn concurrent_appends_produce_unique_seqs() {
    let log = Arc::new(ActionLog::new());
    let mut handles = vec![];
    for _ in 0..100 {
        let log_cloned = Arc::clone(&log);
        let handle = thread::spawn(move || {
            log_cloned.append("SetSeed", ActionPayload::SetSeed { seed: 1 });
        });
        handles.push(handle);
    }
    for h in handles {
        h.join().expect("thread join");
    }
    let entries = log.entries();
    assert_eq!(entries.len(), 100);
    let mut seqs: Vec<u64> = entries.iter().map(|e| e.seq).collect();
    seqs.sort_unstable();
    for (i, s) in seqs.iter().enumerate() {
        assert_eq!(*s as usize, i + 1);
    }
}
