When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Roadmap "Push 9.3 to 9.4" and add the follwing as the new 9.3: 
    1. MORE TOKENS and card variations. 
    1. In an effort to expand the range of good and bad cards, then we need to add a bunch of more tokens and CardEffects that manipulates with these tokens. This is just the beginning of a greater work and there will be adjustments for the future. Here is the list of improvements: 
        1. Every deck that uses a hand should have a deck specific token than controls the max handsize. 
            1. In the future there can be effects that would impact this max handsize. 
            1. This handsize token should be respected. 
        1. Every encounter that has a enemy hand size also has a token controlling that max handsize. 
            1. In the future the player could be able to 
            1. This handsize token should be respected. 
        1. In general: All CardEffects that gives a ressource back (like stamina, shield, dodge etc.) should also have a "max threshold" (or "cap")
            1. Example: A player ressourcing card could give 20 stamina, but the max threshold is 100, so if adding 20 stamina tokens would let the player get above 100 tokens, then it is instead capped at a 100.
                1. The numbers are just examples of the pricniple and not a guide for implementation. 
            1. Make sure there is at least two types of ressourcing cards in the player deck, one with high threshold and one with a low one. 
                1. They could both link back to the same CardEffect with a range. 
                1. If a card have a high max threshold it should not give much ressource and visa versa: if it have a low threshold it should give more. 
        1. Fishing diciplin: 
            1. Fishing player Cards can have multiple CardEffects that have multiple numbers each and any number that would "win the round" would be choosen: if the card is being played. 
                1. The initial player deck only have one card effect pr. card, so there is only one number on each. 
            1. The fishing encounter state struct should move more of its fields to a token based setup. 
            1. Add a new player fishing CardEffect that can remove a "min value"-token from the fish encounter (not below zero) and now that will be used for current turn "win evaluation" and future evaulations.
                1. Also add a similar effect for increasing the "max value"-tokens.
                1. They both have a min and max amount of affected tokens between 50-250. 
                1. Add a couple of cards with this effect to the player deck. 
            1. Add a new player fishing CardEffect that as a "cost" does the reverse of the above, so they narrow the field of min or max. 
                1. They then also have 3-5 values on a single card effect. 
                1. What can affect the cost is: 
                    1. The sum distance between the numbers addded because if they cover a more wide range then it gives more options. 
                    1. The amount of numbers also have some impact, but it is less than the sum distance. 
                1. The cost is defined as a range and can still be a percentage. 
                    1. In this case it just have to be a quite high percentage. 
                    1. A very good card should cost 200-350 and a bad card 50-150. 
                    1. Make some calcualtion that achieves this calculation as a min-max percentage on the card effect. 
                1. Add one card to the player deck with this. 
            1. Add a "fish amount" to fish encounter, as a token. 
            1. Add a player card effect that can increase the fish amount. 
            1. Add a player card effect that have multiple numbers but also decreases the fish amount. 
                1. similar calculation as above. 
            1. Add a player card effect that can give significant amount of stamina, but have no numbers. 
                1. They are a rest action relaxing doing fishing. 
            1. Also, if there is not already then also add a fishing action that costs stamina and have multiple numbers. 
                1. Similar to above. 
        1. Herbalism disciplin: 
            1. Add a player card effect for the following: 
                1. A CardEffect that costs either Stamina or "reward amount" can have higer amount of X in the CardEffects mentioned below. 
                    1. Again on the CardEffect it is a percentage range based on the benefit of the card. 
                1. A CardEffect that removes the type that is present on most cards. 
                    1. But limited to X plant types
                    1. If there is a tie then pick at random. 
                    1. More types are better in the case.
                1. A CardEffect that removes the type that is present on least cards. 
                    1. But limited to X plant types
                    1. If there is a tie then pick at random. 
                    1. More types are better in the case.
                1. A card effect with multiple types, but only matching cards that have all the types (so "and" based)
                    1. At least two plan types and at most all minus one. 
                    1. More types are worse 2 types is best. 
                    1. The same type of CardEffect but "or" based. 
                        1. Same costs etc. 
                1. The simple cards with the simple CardEffect of just one type should be dominant in the initial deck. 
                    1. There can be one of each of the more special CardEffects on different cards, in a moderate version. 
        1. Woodcutting: 
            1. Good/bad cards are quite straigh forward. 
                1. The more numbers and patterns on a card the better it is.
                1. So the sum of different numbers and patterns is the total "benefit".
            1. So CardEffect costs could be stamina and reward amount. 
            1. No cost CardEffect have a "card benefit" between 1-4. 
            1. CardEffects that have a costs will have a total card beefit on 5-15. 
            1. Most of the cards in the initial deck have no costs, but add a couple with moderate costs and moderate beneifts.
        1. Mining: 
            1. Godd/Bad is quite clear: high damage is always good, high defence is usually good. 
            1. Add a future roadmap step on improving the mining gamplay because at the moment it just is a much simpler combat, which is a a bit boring. 
                1. but it is all fine for now. 
            1. Make sure there are cost based CardEffects for stamina and rewards. Like all the other diciplins. 
            1. Make a mix of initial deck with mainly no cost cards and some cost cards. 
        1. Combat 
            1. Add a "milestone insight" reward to all combat. 
                1. Add to the milestone roadmap step that a cost to start a milestone enconter is "milestone insight". 
                1. "milestone insight" (like all other rewards) are a token that is accumulated for the player. 
            1. Add CardEffects like all the other diciplins: That have costs of stamina and rewards for a greater effect and let the deck be mainly not cost effect cards. 
    1. If a card have multiple card effects, then they are evaluated in issolation first to last. 
        1. This can be relevant in many different diciplins.
        1. If a later CardEffect cannot pay its cost, then the previous CardEffects still applied and the card play did not fail. 
            1. In some cases it means that a previous effect could give the ressource that a later CardEffect woul use to pay its cost. 
    1. All cards (except card effects) that have a cost should make the costs in an Vec of (token, amount)
        1. so logic regarding costs can be generalized. 
        1. Any card that cannot pay its cost cannot be played. 
            1. If the enemy picks a random card from hand then it should only try to pick from cards where it can pay the cost. 
                1. If it cannot pay the cost on any of the cards on hand, then pick at random an pay as much of the cost as possible, even if it is zero amount. 
                    1. Add a step later in the Roadmap to expand the enemy AI and emphasise fixing this. 
    1. If I in the above missed something that also could be a token then please ask me if it should be a token. 
        1. Also ask if there should be a matching CardEffect or if manipulating it should be added to the "CardEffect ideas" later. 
1. Roadmap: Modify the "rest encounter step": So that choosing to rest to get stamina costs a mix of herbs and fish. 
    1. So make one Card effect giving a great amount of stamina: With a cost of both fish and herbs, each having its own cost percentage min and max. 
    1. Also make another CardEFfect that is similar but gives Health. 
    1. Also make another that gives a mix og both health and stamina: They each have a min-max and is set at a point where they generally should give less sum as the two other, just to give a benefit to specialization. 
    1. Remember that they give a great amount of tokens, but they they also have a "cap". 
        1. All CardEffects that gives tokens should have a "cap". 
    1. The rest deck should contrary to all the other decks mainly be filled with Cards that have a cost, and only some few that does not have a cost. 
1. Add a section at the end of the roadmap, called "CardEffect ideas": 
    1. Start the section that a lot of these could be introduced with a Milestone boss. 
    1. Ideas to add: 
        1. All diciplins could have a "life steal" Card effect, that improves results using life steal. 
        1. Some milestone can increase the max handsize in a specific diciplin. 
        1. Some merchants can give rare interesting deals: where some permanent toens can be exchanged for other permanent tokens.
            1. Like trade max handsize in woodcutting for max handsize in crafting etc.
        1. Herbalism: Som kind of "guard" token that can protect cards in some way, but only exists for a very small period of time.    
            1. It should not be a guranteed win condition: Maybe something like, the next card will leave half of the cards that would have been removed, at random.
        1. Some CardEffect can make you forget a full card including all counts of the card: 
            1. As long as all the cards are in the library and not in deck, hand, discard etc. 
            1. More powerfull effect the more cards are crafted. 
            1. Still a good effect just because it is ressearced without any crafted cards. 
            1. This is a way to cleanup the library of old unused cards. 
                1. The id in the library should be reusable. 
                1. So would require implmenting empty entries in the library vector and be able to reuse them for new cards. 
                1. It is critical that all existing cards keep their ids! 
        1. Magic CardEffects for all dciciplins. 
        1. Cooking mechanic to make rest encounter more interesting. 
        1. Expand milestones to be faction mechanic with more sense of choices. 
            1. Maybe even a faction diciplin deck.
        1. Expand scouting step to not just deterministicly become harder and give the user more choice in this. 
            1. It should be possible to shape the difficulty (and rewards) of an encounter and then leave the nature of the enemy somewhat random. 
            1. Maybe choosing 1 out of 3 is fine to achieve this?! 
            1. Adding a Scouting diciplin deck when I figured out a good mechanic. 
1. Roadmap: 11 post-encounter scouting:
        1. Rename this step to "simple post-encounter scouting".
        1. Always happens after an encounter is concluded.
        1. It always modifies the encounter card just concluded.
        1. Any mention of tiers here will be postponed to the milestone step.
        1. The player will always be presented with X options: Generate X new encounters of the same type as the encounter just played.
        1. Generate one encounter by:
                1. Every encounter where one or more enemy decks are involved on the enemy side then:
                        1. Keep the number of cards and card counts for each card.
                        1. Pick one card and reroll that card where "affix-pool size" is "CardEffect pool size": where each card have that amount of CardEffects.
                                1. Remember to not change the card type.
                                1. Remember to respect the CardEffect tags relative to the Card type.
                        1. This is very similar to the player crafting step: just done repeatetly and with no player interaction. 
                        1. So if the deck had three cards and each card had 5 copies in total: then only one of the three cards changed and it still has 5 copies.
                        1. It is okay to give the enemy CardEffects for a ressurce that they cannot regain, because the enemy will always start with an intial amount of that ressource.
                1. If there are any numerical values then random change them in the range of -5% to +10%
                        1. So a good chance it will be tougher next time. 
                        1. Here we talk mainly about initial tokens of the encounter. 
                            1. Mainly because everything should be migrated to either being a token or a Card. 
                        1. If there is a min-max value then min should still be less or equal to max.
        1. Let the player choose which of the X encounters to replace the just played encounter.
                1. The player has to choose and cannot keep the just played encounter.
        1. When the player have choosen:
                1. Then replace the current player encounter card with the new encounter card.
                1. Move to the next phase in the encounter.
1. Roadmap: Add to the crafting step. 
    1. Here stamina an healt tokens can be used in CardEffects with costs, for a greater impact. 
        1. There should be the same mix as with the other diciplin cards of the intial deck. 

The following are not pure roadmap changes, but changes to the code. 

1. The "Shield" token does not seem to do much right now, but it should prevent damage. 
    1. The difference between "Shield" and "Dodge" is that dodge is based on timing and can block more damage: While "Shield" is there during the rest of the encounter but then block less (the CardEffects range is smaller, Shield tokens still block 1-1 Health tokens like dodge). 
1. The player starts with 1000 stamina. 


# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
