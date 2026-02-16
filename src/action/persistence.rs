use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::library::types::ActionEntry;

#[derive(Clone, Debug)]
pub struct FileWriter {
    // Shared optional sender so close() can take the sender and drop it.
    sender: Arc<Mutex<Option<Sender<ActionEntry>>>>,
    // Keep a handle to the writer thread so it doesn't get dropped
    _handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl FileWriter {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel::<ActionEntry>();
        let sender = Arc::new(Mutex::new(Some(tx)));
        let handle = thread::spawn(move || {
            // Open file in append mode
            let file = OpenOptions::new().create(true).append(true).open(&path);
            if let Err(e) = file {
                eprintln!(
                    "ActionLog FileWriter: failed to open file {:?}: {}",
                    path, e
                );
                return;
            }
            let file = file.unwrap();
            let mut writer = BufWriter::new(file);
            for entry in rx {
                match serde_json::to_vec(&entry) {
                    Ok(mut bytes) => {
                        bytes.push(b'\n');
                        if let Err(e) = writer.write_all(&bytes) {
                            eprintln!("ActionLog FileWriter: write_all failed: {}", e);
                        }
                        if let Err(e) = writer.flush() {
                            eprintln!("ActionLog FileWriter: flush failed: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("ActionLog FileWriter: serde_json::to_vec failed: {}", e);
                    }
                }
            }
            // rx closed, flush and exit
            let _ = writer.flush();
        });

        Ok(FileWriter {
            sender,
            _handle: Arc::new(Mutex::new(Some(handle))),
        })
    }

    pub fn send(&self, entry: ActionEntry) {
        // best-effort send; ignore failures (e.g., receiver dropped)
        let guard = self.sender.lock().unwrap();
        if let Some(tx) = &*guard {
            let _ = tx.send(entry);
        }
    }

    /// Close the writer: drop the sender and join the writer thread to ensure pending writes flushed.
    pub fn close(&self) {
        {
            let mut guard = self.sender.lock().unwrap();
            *guard = None;
        }
        let handle_opt = {
            let mut h = self._handle.lock().unwrap();
            h.take()
        };
        if let Some(h) = handle_opt {
            let _ = h.join();
        }
    }
}
