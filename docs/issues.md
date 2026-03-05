When the below point states "Roadmap" it means edit the roadmap.md directly.

## Resolved
1. ~~Add a 1000 health tokens to the player when initializing the game.~~ Done: Health: 1000 added in `new_with_rng()`.
2. ~~The rest encounters 1-2 rest tokens, should be a rest encounter card effect.~~ Done: `RestDef { rest_token_min, rest_token_max }` on `EncounterKind::Rest`.
3. ~~Fix pre-existing test failures.~~ Done: Dynamic card ID discovery via API in resolve_play_tests.rs and flow_tests.rs.
4. ~~Check if some of the "Code architecture improvements (future)" and "Known game design gaps (future)" is already implemented.~~ Done: Marked resolved items in roadmap.md.

## Remaining
- **Statistical testing for woodcutting patterns:** The woodcutting multiplier rebalance was calibrated using an external Python Monte Carlo simulation. Consider adding a Rust-native test or benchmark that validates pattern probabilities are within expected ranges, ensuring future deck composition changes don't silently break the probability assumptions.

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear.

Also, make general improvement to both files.
