1. Roadmap step 9.2 change: In the future there will be other costs and maybe multiple costs. 
    1. So make sure to implement step 9.2 with that in mind.
1. Roadmap step 9.3: Also add some cards that can generate durability for a specific Durability type for a certain cost in the gatering materials: wood and ore. 
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
        1. 8 tokens: Craft a new card 
            1. Choose one player card that already exists in the library and try to make a copy of it. 
    1. Crafting encounter: 
        1. Implement the crafting card type
        1. The crafting gameplay is:
            1. The game evaluates the "cost" of the card in gathering tokens. 
                1. Every player card in the library calculates this cost when it is created and persist it on the card af one field to inspect. 
            1. The  

# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 

Also, add a section about the copilot instruction file: for more code style kind of instructions suggestions. 
