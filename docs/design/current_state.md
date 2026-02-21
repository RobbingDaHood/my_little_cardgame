Based on vision.md and roadmap.md, this file describes the current state of the project. 

Step 1 of the roadmap is currently being implemented and refined.

Step 3 (ActionLog & structured actions API) progress:

- Implemented: in-memory append-only ActionLog with structured ActionEntry (timestamp, actor, request_id, version)
- Implemented: GameState::append_action central append API
- Implemented: durable writer queue (FileWriter) with bounded queue, batching, and optional fsync (ACTION_LOG_FSYNC)
- Implemented: single writer thread with close() and GameState::shutdown() which flushes pending writes
- Implemented: GET /actions/log with pagination/filters and ActionLogResponse including next_seq
- Implemented: POST /action now returns appended ActionEntry as 201 Created JSON
- Tests: replay, concurrency, and persistence tests added and all pass locally

Remaining:
- Migrate ActionLog internals to async-aware non-blocking locks to avoid blocking Rocket's async runtime (recommended)
- Improve durability semantics (fsync batching, error handling, retries) as required for production
- Update OpenAPI docs and Swagger UI (some route signatures changed; re-generate API docs if needed)
- Push feature branch and open PR for review (branch exists locally; remote push requires credentials)
 
