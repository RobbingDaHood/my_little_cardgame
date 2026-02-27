1. The players Durability tokens are initialized at the start of the game.
    1. At the moment there are no way of getting them back when they are lost. 
    1. Later steps then they can be regained with crafting. 
    1. You can change the initial Durability to a 100, just to have something to work with. 
        1. The scenario test can then keep on playing mining encounters until the player looses. 
1. The Durability is diciplin specific: so the current implemented durability should have a subtype of Mining. 
1. The MiningEncounterState should use tokesn instead of a ore_hp field.
    1. There is no need for a "ore_max_hp" 
1. Mining does not have any penalties: If you durability is used up then you loose the encounter. 
1. There does not seem to be any need for "last_durability_prevent": 
    1. When a player chooes a card, then the enemy chooses one straight away too: so there are no need to store the last_durability_prevent in the MiningEncounterState.
1. The "rewards" should be (token, amount) not just (token_type, amount). 
1. It should be possible to abort any non-combat encounter: Marking the encounter as lost, going to the post-encounter scouting phase directly.
    1. This is a new PlayerAction
    1. The penalty would still apply if aborting.
1. The enemies CombatCardCounts and OreCardCounts are the same: generalize that and use both places. 
1. MiningEncounterState.encounter_car_id is not optional but mandatory. 
1. Both for MiningEncounterState and CombatEncounterState: There is no need to have a "is_finished" when there is a EncounterOuctome.UNDECIDED, undecided means "not is_finished".
1. The EncounterPhase.Combat and EncounterPhase.Gathering should be merged into one state: something like "InEncounter". 

# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 

Also, add a section about the copilot instruction file: for more code style kind of instructions suggestions. 
