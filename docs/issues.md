1. **Replay system is incomplete**: `replay_from_log` currently only handles legacy token operations (GrantToken, ConsumeToken, ExpireToken). It should be extended to replay player actions (NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting) to achieve the vision's goal of full game state reproduction from seed + action log.

2. **Token registry entries (TokenRegistryEntry) could use cleanup**: The `TokenRegistryEntry` struct still has `lifecycle` and `cap` fields, but lifecycle is now dynamic on Token instances. Consider whether the registry entry's lifecycle is a "default lifecycle" or if it should be removed.

4. **JSON serialization of Token maps is verbose**: The array-of-entries format (`[{token: {token_type, lifecycle}, value: N}, ...]`) works but is verbose. Consider whether a more compact serialization (e.g., `"Health:PersistentCounter": 20`) would be better for API consumers.

5. **CombatSnapshot could be further simplified**: With player tokens now external, CombatSnapshot only tracks enemy state and combat metadata. Consider whether it should be renamed to reflect this (e.g., `CombatState` or `EnemyCombatState`).

6. When the enemy picks a random card then it should not be picked from the deck, but from the hand. Each of the cards in the enemy deck should have similar counters as the counters in the Lirrary with some changes. 
    1. There ar only deck, hand and discard counts. 
    1. At the start of every combat the "hand" is randomized in each of the decks: meaning that first take the total count of cards on hand in the deck currently, then "put all the cards in the deck" by changing the counts accordingly. 
    1. Then pick random cards fromt he deck until the hand is full again. 
    1. This random draw only happens at the start of a combat. 
    1. From this point on the enemy only draws cards if they play a ressource card with the draw card effect. 
    1. All card that are played goes to the discard pile. 

1. If any "deck" is empty then move the discard pile to the deck: by changing the counts. 

1. Whn the ressource card effect triggers then take a random card from "the deck" and put it in "the hand": by changing the counts. 

1. Just get tid of this "with_default_lifecycle" and hardcode the initial durations: 
    1. Most tokens are hardcoded to PersistentCounter 
    1. Dodge is the only exception, that is just until the next Defence phase in the combat: so something like: TokenLifecycle::FixedTypeDuration { duration: 1, phases: vec![EncounterPhase::Defence], }
    1. So they are all persisted between combats. 
    1. Card effects should also explicit state the duration of the token they provide. 









# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 
