use super::types::{ActionEntry, ActionPayload};
use crate::action::persistence::FileWriter;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[derive(Debug)]
pub struct ActionLog {
    pub entries: Arc<Mutex<Vec<ActionEntry>>>,
    pub seq: AtomicU64,
    pub sender: mpsc::Sender<ActionEntry>,
    pub writer: Option<FileWriter>,
}

impl Clone for ActionLog {
    fn clone(&self) -> Self {
        // snapshot existing entries and seq
        let entries_vec = match self.entries.lock() {
            Ok(g) => g.clone(),
            Err(e) => e.into_inner().clone(),
        };
        let seq_val = self.seq.load(Ordering::SeqCst);
        // create a fresh ActionLog (spawns its own worker)
        let new = ActionLog::new();
        // replace entries with the snapshot
        match new.entries.lock() {
            Ok(mut g) => *g = entries_vec,
            Err(err) => *err.into_inner() = entries_vec,
        }
        new.seq.store(seq_val, Ordering::SeqCst);
        Self {
            entries: new.entries,
            seq: new.seq,
            sender: new.sender,
            writer: self.writer.clone(),
        }
    }
}

impl Default for ActionLog {
    fn default() -> Self {
        ActionLog::new()
    }
}

impl ActionLog {
    pub fn new() -> Self {
        let entries = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = mpsc::channel::<ActionEntry>();
        let _worker_entries = Arc::clone(&entries);
        thread::spawn(move || {
            // Dedicated worker receives entries for offloaded processing (persistence, analytics, etc.).
            // Currently it consumes the channel and drops entries after receipt to keep the worker alive
            // without duplicating in-memory storage (append() writes directly into entries).
            for _entry in rx {
                // placeholder: persist or forward the entry to external systems
            }
        });
        ActionLog {
            entries,
            seq: AtomicU64::new(0),
            sender: tx,
            writer: None,
        }
    }

    pub fn set_writer(&mut self, writer: Option<FileWriter>) {
        self.writer = writer;
    }

    pub fn load_from_file(path: &str) -> Result<ActionLog, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut max_seq = 0u64;
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: ActionEntry = serde_json::from_str(&line).map_err(|e| e.to_string())?;
            if entry.seq > max_seq {
                max_seq = entry.seq;
            }
            entries.push(entry);
        }
        let log = ActionLog::new();
        {
            match log.entries.lock() {
                Ok(mut g) => *g = entries,
                Err(e) => *e.into_inner() = entries,
            };
        }
        log.seq.store(max_seq, Ordering::SeqCst);
        Ok(log)
    }

    pub fn write_all_to_file(&self, path: &str) -> Result<(), String> {
        let entries = self.entries();
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        for e in entries {
            let line = serde_json::to_string(&e).map_err(|e| e.to_string())?;
            writeln!(f, "{}", line).map_err(|e| e.to_string())?;
        }
        f.flush().map_err(|e| e.to_string())
    }

    /// Append an action entry, assigning an incrementing sequence number.
    /// This implementation writes into the in-memory entries immediately (synchronously)
    /// and also sends the entry to a background worker for offloaded tasks.
    pub fn append(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
        self.append_with_meta(action_type, payload, None, None, None)
    }

    pub fn append_with_meta(
        &self,
        action_type: &str,
        payload: ActionPayload,
        actor: Option<String>,
        request_id: Option<String>,
        version: Option<u32>,
    ) -> ActionEntry {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst) + 1;
        let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => format!("{}", dur.as_millis()),
            Err(_) => "0".to_string(),
        };
        let entry = ActionEntry {
            seq,
            action_type: action_type.to_string(),
            payload: payload.clone(),
            timestamp,
            actor,
            request_id,
            version,
        };
        // write into in-memory entries immediately to preserve synchronous semantics
        match self.entries.lock() {
            Ok(mut g) => g.push(entry.clone()),
            Err(e) => e.into_inner().push(entry.clone()),
        }
        // best-effort send to worker for offloaded processing; ignore errors if the worker has shut down
        let _ = self.sender.send(entry.clone());
        entry
    }

    /// Return a cloned snapshot of entries for replay/inspection
    pub fn entries(&self) -> Vec<ActionEntry> {
        match self.entries.lock() {
            Ok(g) => g.clone(),
            Err(e) => e.into_inner().clone(),
        }
    }

    /// Async wrapper for compatibility with async callsites.
    pub async fn append_async(
        self: Arc<Self>,
        action_type: &str,
        payload: ActionPayload,
    ) -> ActionEntry {
        // append is non-blocking (sends to worker) so this can be synchronous
        self.append(action_type, payload)
    }
}
