The follwing is a list of cleanups that need to be performed. You do not need to do it in the order of writing and you should plan them in the order that makes the most sense. 

# The library is missing!

Now I see a major reason for some design drift in the code and this is a very high priority to fix. 

The library does not even exist! 

THere is a "/library/cards" endpoint that make a lookup in player_data.cards, but this is wrong. 

The "GameState" should contain a "Library" that is an array of cards: the index of the array is the ID of the card. All cards that does not belong to an enemy is in this library, no matter the type etc. 

The Library defines the state of all the cards of a certain type: as explained in the vision under "card location model and counts": BTW this section is authoritive when it comes to anything that mentions card counts and placement, so suggest change to the vision.md wherever this section is contradicted. 

# All card ids is the index in the library 

So it is fine to still state the IDs as long as they refer back to their index in the library and can be used for a lookup. 

# Cards do not solely express combat cards

There are "combat encounter"-cards (read below), attack cards, defence cards, ressource cards, encounter cards etc. 

In the future there will be way more cards. 

So the CardDef need to represent this flexibility. 

Likely change the "card_type" and "effect" with some enum that can contain the payload relevant for that enum.

# Encounters are a card too in the library 

They have the type "encounter". 

They have no "entry_cost", "state", affixes". 

In case of a combat encounter (The only one that exists at this stage in the roadmap) then the "Combat encounter" does define a full comabant: 
1. Initial tokens
1. The three decks (attack, deffence, ressourcing) that defines the unit, with all the cards in each. (Enemies are the only ones that defines an actual deck explicitly). 

You can remove the "reward_deck_id" too, in a later step we will implement rewards. It will be a number of cards that can be drawn from the combat reward deck, but that is for furture implementation. 

# There are still decks being made and that is not allowed. 

This is a fundamental change and VERY important. 

All cards are in the library and is solely represented there. 

Decks do not exist explicitly, only implicitley as all cards of the same type in the library belongs to the same deck. 

It is fine to have helper functions that help interact with library: But there are no decks explicit in the data, it all refers back to the library. 

The only exception to this is the combat encounters mentioned above. That deck setup should be close to the combatant models and clearly express that they should only be used in that context. 

# TokenLifecycle.TokenLifecycle need to define what EncounterPhase it counts. 

So it should keep the u64 but also add an enum. 

The idea is that the duration should count down every time we get to a specific phase. 

Better yet, make it a list of phases: in case we want to count down in multiple different phases. 

# The combatant does not need an ID

The combatant is always the current combat encounter and never the player. 

It just need current tokens and current decks state (only exception for decks in the whole code base). 


# CombatState is reduntant. 

The full state can be represented with player and enemy tokens and cards + the EncounterPhase.

So remove CombatState. 

# Token registry 

Use the token type as the key and the hashmap then maps to a count of that type of token. 

# Overall: 

Everything should much more refer to the "library" when it comes to the player cards. 

The cards are much more flexible and can represent a lot of different cards. 

The tokens represent current state of the player and if in a combat the combatant. 


# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 
