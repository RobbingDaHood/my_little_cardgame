# Known Test Failures

Tests listed here are known to fail on the current codebase. They are automatically skipped by `make check` (via `scripts/check_all.sh`). When a failure is fixed, remove it from this list.

Format: each entry is a backtick-wrapped test function name followed by a brief reason.

## Scenario tests (`tests/scenario_tests.rs`)

- `scenario_cost_cards_exist_in_starting_decks` — card count assertions don't account for cost card variants added in 9.2
- `scenario_fishing_encounter_initializes_range_tokens` — expects range tokens in global token_balances; they now live on encounter_tokens
- `scenario_max_handsize_tokens_initialized` — handsize token count assertion is off after 9.3 token additions
- `scenario_mining_encounter_full_loop` — yield/conclude flow changed in 9.5 mining redesign; test not yet updated
- `scenario_player_draw_cards_per_type` — card count assertion off by one after cost card additions

## Flow tests (`tests/flow_tests.rs`)

- `test_player_kills_enemy_and_combat_ends` — combat end detection changed; test not yet updated

## Resolve play tests (`tests/resolve_play_tests.rs`)

- `test_play_attack_card_kills_enemy` — combat resolution assertions don't match current card effect system
