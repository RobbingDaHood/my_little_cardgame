use my_little_cardgame::action::persistence::FileWriter;
use my_little_cardgame::library::types::{ActionEntry, ActionPayload, TokenType};
use std::io::Read;

#[test]
fn file_writer_writes_and_flushes() {
    let dir = std::env::temp_dir().join(format!(
        "my_little_cardgame_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_action_log.jsonl");

    let writer = FileWriter::new(path.clone()).expect("create writer");

    // Send a few entries
    for i in 0..3 {
        let entry = ActionEntry {
            seq: i,
            action_type: "GrantToken".to_string(),
            payload: ActionPayload::GrantToken {
                token_id: TokenType::Insight,
                amount: (i as i64) + 1,
                reason: None,
                resulting_amount: 0,
            },
            timestamp: format!("{}", i),
            actor: None,
            request_id: None,
            version: None,
        };
        writer.send(entry);
    }

    // Close to flush
    writer.close();

    // Read back the file
    let mut contents = String::new();
    std::fs::File::open(&path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    // Should have 3 lines
    let lines: Vec<&str> = contents.lines().collect();
    assert_eq!(lines.len(), 3);

    // Each line should be valid JSON
    for line in &lines {
        let _: ActionEntry = serde_json::from_str(line).expect("valid JSON");
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn file_writer_close_is_idempotent() {
    let dir = std::env::temp_dir().join(format!(
        "my_little_cardgame_test2_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_action_log2.jsonl");

    let writer = FileWriter::new(path.clone()).expect("create writer");
    writer.close();
    writer.close(); // idempotent

    // Send after close should not panic
    let entry = ActionEntry {
        seq: 0,
        action_type: "GrantToken".to_string(),
        payload: ActionPayload::GrantToken {
            token_id: TokenType::Health,
            amount: 1,
            reason: None,
            resulting_amount: 0,
        },
        timestamp: "0".to_string(),
        actor: None,
        request_id: None,
        version: None,
    };
    writer.send(entry);

    let _ = std::fs::remove_dir_all(&dir);
}
