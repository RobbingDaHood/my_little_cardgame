use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::library::types::ActionEntry;

#[derive(Clone, Debug)]
pub struct FileWriter {
    // Shared optional sender so close() can take the sender and drop it.
    sender: Arc<Mutex<Option<SyncSender<ActionEntry>>>>,
    // Keep a handle to the writer thread so it doesn't get dropped
    _handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl FileWriter {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        // Configure queue size and fsync behaviour using environment variables
        let queue_size: usize = std::env::var("ACTION_LOG_QUEUE_SIZE")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(1000);
        let fsync_enabled: bool = std::env::var("ACTION_LOG_FSYNC")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let (tx, rx) = mpsc::sync_channel(queue_size);
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

            let mut buffer: Vec<String> = Vec::new();
            loop {
                // Block waiting for the first entry
                match rx.recv() {
                    Ok(entry) => match serde_json::to_string(&entry) {
                        Ok(line) => buffer.push(line),
                        Err(e) => {
                            eprintln!("ActionLog FileWriter: serde_json::to_string failed: {}", e)
                        }
                    },
                    Err(_) => {
                        // Sender was dropped, flush remaining and exit
                        if !buffer.is_empty() {
                            for ln in buffer.drain(..) {
                                let _ = writeln!(writer, "{}", ln);
                            }
                            let _ = writer.flush();
                            if fsync_enabled {
                                let _ = writer.get_ref().sync_data();
                            }
                        }
                        break;
                    }
                }

                // Batch additional available entries (non-blocking)
                while let Ok(entry) = rx.try_recv() {
                    if let Ok(line) = serde_json::to_string(&entry) {
                        buffer.push(line);
                    }
                }

                // Write batch
                if !buffer.is_empty() {
                    for ln in buffer.drain(..) {
                        let _ = writeln!(writer, "{}", ln);
                    }
                    if let Err(e) = writer.flush() {
                        eprintln!("ActionLog FileWriter: flush failed: {}", e);
                    }
                    if fsync_enabled {
                        if let Err(e) = writer.get_ref().sync_data() {
                            eprintln!("ActionLog FileWriter: sync_data failed: {}", e);
                        }
                    }
                }

                // Small sleep to avoid busy-looping if entries arrive intermittently
                thread::sleep(Duration::from_millis(5));
            }
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
            // If the channel is full, this will block and provide backpressure
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
