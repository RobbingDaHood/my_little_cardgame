# Use more enums 

1. Combatant.active_tokens is a hashmap from String to i64: It should use a Enum for the tokens (instead of the string).
    1. Bomnus: it should also be u64 instead of i64.
1. CardEffect.token_id: Should also be a Enum of token types, with their own payload. 
1. CombatantDef.initial_tokens: The same as above. 
1. CardDef.card_type should be an enum. 

The token enum should at least contain TokenLifecycle beside its name. 

See vision.md about some of the tokens that should be created and the roadmap.md on an indication when they should be created. 

# ScopedToEncounter is just FixedTypedDuration

Something like: FixedTypeDuration(EncounterPhase::Scouting, 1) so the first time we get to Scouting the token duration is up. 

So no need to make a new token lifecykle. 

So delete: FixedTypeDuration


# Fix github build errors

The full log can be read at docs/full_github_actionlog.txt 

Analyse that and fix the issue. 

# The player cannot abbandon the combat. 

Maybe we will in the future make a "flee" card, but there is no way for the player to abbandon a fight. You fight until either the enemy or you are dead. 

# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 
