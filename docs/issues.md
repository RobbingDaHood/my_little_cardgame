When the below point states "Roadmap" it means edit the roadmap.md directly.

1. The initial decks should also be populated from the relevant diciplin files. 
    1. Example: all cards that relates to mining should be initialize in the mining file. 
    1. All CardEffects used in multiple diciplins can stay in the GameState file. 
1. Consider if all the "all_X_hand_cards_unpayable" methods cannot be generalized? 
1. Consider if the "X_play_random" methods cannot be generalized: Seems like it is always about picking a random card from a hand for cards with a spcieific type. 
    1. Same with drawing cards. 
    1. And shuffle hands
    1. And fetching discard into the deck
    1. And draw random 
1. Add a step as the last 9.x step: Better mining. 
    1. Mining is about: 
        1. Maintaining a light level 
        1. While doing the mining 
        1. And saving energy for carrying all the yield
    1. So the player can conclude the game at any point, and then he gets the lowest of stamina and yield: That will cost that amount of stamina to conclude. 
    1. In addition there are player CardEffects that increases the "light level": 
        1. This is a set of tokens starting at 300 at the start of the encounter. 
        1. Every card the enemy plays can have the durability loss as of today and also reduces the light level with a moderate amount. 
            1. So they both have CardEffects that only does one or the other, but most will do both. 
            1. The ones that only do one does more of it then.
        1. The Player have seperate card effects that increases the light level with a high amount: but not a CardEffect that has both. 
            1. Later in the game when the player starts crafting cards with multiple card effects then one card could have both. 
    1. Instead of the enemy only having X health, then they have no health and the player cannot win by reducing the non existent health. 
        1. The player can only win by ending the game. 
        1. The player can loose by running out of durability or not being able to play any cards. 
    1. When the player plays a "damage" card (renamed to just mining and the token on the card effect is now called "mining power") then a "yield" token is accumulated: 
        1. It is the "mining power" * "light level" / 100 
    1. Because there are no enemy then the enemy cannot have any CardEffects that cost stamina. 
    1. The enemy does have some rare cards that removes a bit of the players health.
        1. That card effect is rare and does not take a lot of life. 
1. The woodcutting multiplier on matching patterns need to have a higher difference. 
    1. The difference should be proportional with the statistical probability of getting that pattern. 
    1. Do not make it dynamic based on the number of turns, cards etc. Just assume 8 cards player out of 13 cards total, what is the probability of a pattern. 

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
