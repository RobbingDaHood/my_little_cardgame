When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Add a 1000 health tokens to the player when initializing the game. 
1. The rest encounters 1-2 rest tokens, should be a rest encounter card effect. 
- **Fix pre-existing test failures:** `test_play_attack_card_kills_enemy` (resolve_play_tests.rs) and `test_player_kills_enemy_and_combat_ends` (flow_tests.rs) both hardcode card IDs 8, 9, 10 that changed during the card initialization refactoring. These need to discover card IDs dynamically via the API (e.g., query `/library/cards?card_kind=Attack`).
- **Statistical testing for woodcutting patterns:** The woodcutting multiplier rebalance was calibrated using an external Python Monte Carlo simulation. Consider adding a Rust-native test or benchmark that validates pattern probabilities are within expected ranges, ensuring future deck composition changes don't silently break the probability assumptions.
1. Check if some of the "Code architecture improvements (future)" and "Known game design gaps (future)" is already implemented: remove the point if it is. 
    1. If the point needs updating then do that too. 

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
