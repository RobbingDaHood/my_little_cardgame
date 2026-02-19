# 0002 - ActionLog locking strategy

Summary:
ActionLog currently uses `std::sync::Mutex` for the entries buffer and `AtomicU64` for seq. This is correct for correctness, but mixing blocking mutexes with async code can cause executor thread blocking under contention.

Tasks:
- Evaluate replacing `std::sync::Mutex` with an async-friendly primitive (e.g., `tokio::sync::Mutex`) or `parking_lot::Mutex` depending on deployment.
- Run stress tests to measure contention after change.
- Decide and implement migration in a small PR if warranted.
