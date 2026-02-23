# The player cannot abbandon the combat. 

The player cannot abandon combat. So remove AbandonCombat and all logic related to it. 

# FinishScouting should not be a player action

EncounterApplyScouting is the action that will "finish" the Scouting post-encounter phase. 

# ApplyScouting is not needed

EncounterApplyScouting is the action that is used in the post-encounter scouting phase. o

# There is no DrawEncounter 

When an encounter have been resolved then new cards are automaticly drawned to the "area deck": at random. 

So the player cannot decide what to draw to the hand. 

So delete EncounterPickEncounter.

# There is no ReplaceEncounter 

Because it is replaced with EncounterPickEncounter.

So delete EncounterPickEncounter

# You do not need GrantToken PlayerAction

Because the player is solely supposed to play cards, it cannot give itself tokens. 

# SetSeed Should just be done at the beginning of the game 

Remove this from player action, but implement a "NewGame" action that can take a optional seed. 

This way, then you can start the server and start a NewGame providing the seed and play on. This will also be used in the tests to ensure all tests use a hardcoded seed. 

If a seed is not provided then genrate a random seed. 

At the start of the server just generate a new game with a random seed. 

# PlayCard is reundant now

Either the player will use EncounterPickEncounter, EncounterPlayCard or EncounterApplyScouting: And then there is no other way to play a card. 

# The handler of EncounterPickEncounter should not assume the encounter is combat

Even though all encounters right now are combat, then do not assume that is the case. So it should check if the choosen card is an encounter and then check if it is a combat encounter. 

# Ensure that all player actions go to the actionlog and only playeractions are on the action log. 

To keep the actionlog simple and the only requirement is that with an actionlog and a seed then the full game state can be reproduced. 


# There are still references to player_data.current_area_deck in the code

There are no explicit area deck in the code anywhere: it is solely represented in the library as all the "encounter cards" that have at least one count in the "deck" counter. 

It is fine if there are helper functions that mentions area deck etc: but they should all point back to the library. 

The current AreaDeck is a list of ids to the library: but it should just not exist. If we want to know the "deck" of the "area", then query the library for all cards of type "encounter" that has at least one count in the "deck count". If you want to get the hand of the "area" then query the library for all encounter cards that has at least 1 in the "hand count". 

This is how all the player decks works. 


# There is a non test POST endpoint: /combat/enemy_play and /combat/advance

There should not be any non test POST or PUT endpoints. 

The enemy should just pick random cards in response to the player picking a random card. It should not be exposed as an endpoint the player can trigger. 

The combat should just advance automaticly.

# Renamed TokenId to TopenType


# The token lifecykle is not static for a specific TokenId

When creating a "Token", it consists of:
1. TokenType (se rename above) 
1. TokenLifecykle 

Then the tokens list on the player and combatant uses the above "token" as a key and maps to a count of them.

Example: 
1. The player start the game with Health, Persistent and 20. Meaning as long as the player does not take damage then he keep having 20 health. 

So create a "Token" type with the two fields and use that in the "maps" from Token to count. 

# CardKind: Resource: draw_count is a specific type of Card effect 

It is not a field of the Ressource card. 

# CardKind: CombatEncounter: Should be a Encounter 

And then Encounters have its own set of EncounterKind enum, where one of them is a CombatEncounter. 

This is to prepare for the future where there are multiple EncounterKinds. 

# Correction: Combat: enemy picking random card. 

The enemy will pick a randomc ard that matches the current CombatPhase. 

So attack cards during the attack phase, etc. So only one card at a time from the correct deck. 

# The player tokens should live outside a CombatSnapshot 

It is part of the game state and not a CombatSnapshot. 

All cards that affect the player tokens should solely change the GameState tokens. 

# The CombatResult is reduntant

So change the CombatSnapshot.winner type to a enum with: UNDECIDED, PLAYER_WON, ENEMY_WON

Delete CombatResult struct. 


# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 
