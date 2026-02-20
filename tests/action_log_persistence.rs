use std::env;
use std::fs;

use my_little_cardgame::library::{action_log::ActionLog, types::ActionPayload};

#[test]
fn write_and_load_actionlog_file() {
    let log = ActionLog::new();
    log.append(
        "GrantToken",
        ActionPayload::GrantToken {
            token_id: "Insight".to_string(),
            amount: 3,
            reason: None,
            resulting_amount: 3,
        },
    );
    log.append(
        "GrantToken",
        ActionPayload::GrantToken {
            token_id: "Renown".to_string(),
            amount: 7,
            reason: None,
            resulting_amount: 7,
        },
    );

    let mut path = env::temp_dir();
    path.push(format!("mlcg_actionlog_test_{}.jsonl", std::process::id()));
    let path_str = path.to_str().unwrap().to_string();

    // write to file
    log.write_all_to_file(&path_str).expect("write file");

    // load back
    let loaded = ActionLog::load_from_file(&path_str).expect("load file");
    assert_eq!(loaded.entries().len(), 2);

    // cleanup
    let _ = fs::remove_file(&path_str);
}
