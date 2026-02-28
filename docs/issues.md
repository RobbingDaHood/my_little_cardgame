When the below point states "Roadmap" it means edit the roadmap.md.

1. Roadmap step 9.2 change: In the future there will be other costs and maybe multiple costs. 
    1. So make sure to implement step 9.2 with that in mind.
1. Roadmap change: 8.4: About fishing: 
    1. The fishing mechanics should change. 
    1. Each fishing mechnics states a "valid range", a max amount of turns and a win amount of turns. 
    1. Each round the player first and then the enemy play a numeric card: The two cards are substracted from each other (can not go below zero). If the sum is within the valid range, then the turn is "won". 
    1. If the player win enough rounds before the max turns are over then the player wins. 
    1. If the max amount of turns are over before the player win enough rounds then the player looses. 
    1. Every card have a small amount of durability loss: that is the second way to loose. 
1. Roadmap 9.1: Major refactor: range system. 
    1. Simplify it a bit. 
    1. CardEffects solely have min and max
    1. "Concrete cards" Always shows a fixed number and not a range. 
1. Roadmap 9: Crafting encounters and diciplin 
    1. A crafting encounter has a certain amount of "Crafting tokens". 
        1. To begin with around 10. 
    1. There are multiple player actions to be done during a crafting encounter that each cost different amount of crafting tokens: 
        1. 1 token: replace a card between the deck or discard pile, with the library. 
            1. So choos two cards: one move from deck/discard to library, and the other does the opposite. 
            1. There need to be cards available. 
            1. You cannot move from the hand. 
            1. You can only do this on the player cards and not area cards. 
        1. X tokens: Craft a new card 
            1. Choose one player card that already exists in the library and try to make a copy of it. 
        1. 1 token; Add durability to a choosen diciplin for a cost of some wood or ore. 
    1. Crafting encounter: 
        1. Implement the crafting card type
        1. The crafting gameplay is:
            1. The game evaluates the "cost" of the card in gathering tokens. 
                1. Every player card in the library calculates this cost when it is created and persist it on the card af one field to inspect. 
                1. The more effects and the better roles the effects have, the higher the cost. 
            1. The game is player over X turns, every turn costs 1 crafting token. 
                1. The player plays a card. 
                1. Crafting cards have one or more gathering tokens and a number for each: every time they play a card they reduce the cost of the craft with what is mentioned on the card. 
                1. The cost can at maximum be halfed in each of the cost token types. 
                1. The enemy have a similar deck and also plays a card every turn that increases the cost of one or more tokens. 
                    1. In general the enemey cards are scewed in a way where the player card is slightly more powerfull: initially.
            1. The player can only loose the encounte if the player cannot pay the final cost else they win it. 
        1. The player can abort a crafting encounter at any point.
1. Roadmap change: Heavy changes to "8.5) Refined gathering encounters" 
    1. Do not chagne any of the phases or number of decks for any of the gathering encounters. Theya re fine as is. 
    1. Also skip "Affix modifiers" for the scouting step later in the roadmap. 
    1. Stamina have replaced rations when it comes to costs. 
    1. In each of the gathering diciplins make Tiered reward tokens: So like wood tier 1, 2 and 3. 
        1. Then add player card effects for each tier, that makes the gathering encounter more difficult, but also increases the tier of the reward. 
            1. It should be somewhat moderat hard to win tier 2 and very hard to win tier 3. 
            1. It should not make it more difficult by just removing some durability, but being more involved in the gameplay of each diciplin.
    1. The insights tokens will also have three tiers. 
    1. The initial decks only have very few cards with the increase tier effects. 
    1. The tiers could be expanded later, but at least for now it is just three. 
    1. "Combos and momentum" will wait for later: maybe until the milestone step. 
    1. Remove the mention of "Tool cards and special extraction" moves. 
    1. Adjust the "Playable acceptance" to fit the above.


# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 

Also, add a section about the copilot instruction file: for more code style kind of instructions suggestions. 
