# Vision & Roadmap Suggestions — Herbalism Balance Tuning

Based on findings from the herbalism balance tuning session.

## vision.md suggestions

1. **Herbalism encounter depth**: The current 8-plant-type config with single/dual characteristics creates a meaningful decision space, but win rates remain low (~14-22%). The "exactly 1 remaining plant" win condition is inherently hard — consider whether partial-win rewards (e.g., 2-3 remaining = reduced reward) would deepen the strategic space without removing the challenge.

2. **Strategy differentiation ceiling**: The greedy single-card-per-round approach is near-optimal for herbalism because Or cards are order-independent and the best greedy play rarely conflicts with the globally optimal sequence. Future encounter designs could introduce order-dependent mechanics (e.g., cards that change plant characteristics mid-encounter) to widen the gap between simple and tactical play.

3. **Cost system interaction**: Free Or cards dominate herbalism strategy because costly cards (And/MostCommon/LeastCommon) have HIGHER durability costs while providing less removal breadth. The cost system creates a clear tier separation (Tactician avoids costly cards → less durability → higher yield/dur) but limits the design space for "spend resources wisely" strategies.

## roadmap.md suggestions

1. **Herbalism balance targets are now met** — update roadmap to reflect Tier 1 (0.3-2.0) and Tier 2 (1.3-4.0) assertions are enforced in CI. The herbalism discipline is ready for gameplay testing.

2. **Consider adding a "loss minimization" mechanic**: Currently, encounters with unwinnable hands still consume full durability because the player must keep playing. An "abort encounter" or "concede early" action could create a new strategic axis — recognising and cutting losses early.

3. **Cross-discipline parity check**: Herbalism yield/durability ratios (~0.5-2.0 for Tier 1, ~2.0 for Tier 2) should be compared against mining, woodcutting, and fishing to ensure gathering disciplines offer comparable progression rates.

4. **Plant characteristic diversity**: The current 5 characteristics (Fragile, Thorny, Aromatic, Bitter, Luminous) with 3 dual-char types provide good variety. Future expansions could add more characteristics or plant types to scale encounter difficulty with player progression.
