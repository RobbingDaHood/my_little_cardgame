1. Change step 8.5 of the roadmap: Add the "insights" card effect. 
    1. It can be used on all types of cards. 
    1. When triggered it gives the diciplin specific "insight" token.
        1. It is used later for the ressearch diciplin. 
    1. Add some cards to every diciplin that has a worse effect, but then give some "insight". 
1. Every play of a herbalism card have a moderate stable durability loss: like around 1. 
    1. There are no defenece against this. 
    1. This is because more Characteristics are not nessesary a good thing.
1. The herabalist starting encounter enemy hand size is 5. 
    1. The hand is randomized at the beginning of the encounter. As with the other encounters. 
1. When aborting a Encounter do not return 400: It is still a success, you succcessfully aborted the encounter. 
1. All the herbalism player cards should just cost 1 durability no matter how 






1. Change the roadmap: Adda a step 9.1 that is a major refactor of the CardEffects setup: 
    1. CardEffects now define the following for each nummeriacal value: 
        1. min min: The minimum value of the min value. 
        1. max min: the maximum value of the min value. 
        1. min max: The min value of the maximum value 
        1. max max: The max value of the maximum value. 
    1. So each CardEffect defines a range of values. 
    1. Then every concrete card (that currently just have a refference to a card effect) is a concrete implementation of this effect. 
        1. So it keep having a reference to the CardEffect. 
        1. In addition it solely have a min and a max. 
        1. This min and max gets decided when the card is created. 
            1. A random role for the min between "min min" and "max min". 
            1. A random role for the max between "min max" and "max max"
        1. When a card is played then it is the concrete min and max on the card that is used. 
        1. This also means that all cards have ranges that are "rolled" when played to a concrete number. 
            1. Whe the value have to be between min and max of the concrete card. 
    1. Remember to always use the game seed for all of these new roles, to make sure the outcome is reproduceable with the seed and action log. 
    1. This also means we can remove some of the current CardEffects: because they are overlapping with each other. 
    1. Later on ressearch will add more CardEffects and crafting will add more concrete cards. 
        1. That is also why we are making this step just before implementing them. 
    1. We likely need to bump most of the numbers on cards and tokens etc. with a factor of 100. 
        1. Just to create more interesting ranges everywhere. 
        1. Because currently some of them just have a number of 1 and it would be to bigg an impact if they then went from 1-10. 
            1. While it would be fine if they were from 80-120. 

1. Change the roadmap: Adda a step 9.2 that is a refactor of the CardEffects setup: 
    1. Some CardEffecs have a cost.
    1. If they have a cost then the cost is part of the CardEffect. 
    1. The cost will be a range of multipliers. 
    1. It is deined as a percentage. 
    1. Example: If a Attack card has health effect range betwee (10-20)-(40-50) and a stamina cost of 20-40. 
        1. Then the concrete card could roll with: 13-45, 25. 
        1. Meaning that when the card is played it rolls between 13-25 and maybe gets 20, 25% of 20 is 5, so it will cost 5 Stamina. 
    1. Make sure that cards that have a cost is significantly better than the ones that does not have a cost. 
    1. In combat there should be a at least one cost variation for each non-cost variation when it comes to attack and defence. 
        1. The cost at the beginning is stamina. 
    1. Woodcutting and mining can also get stamina costs on their cards. 
        1. Same thing, there should be a non-cost card for each cost card. 
        1. Starting deck should mainly start with non-cost cards.
    1. The Stamina used cross diciplin is the same. 
    1. The starting deck should mainly have non cost cards for attack and defence: just to make sure they are easy to play with initially. 

1. Change the roadmap: Add a step 9.3: Add a rest encounter. 
    1. Add a new rest encounter. 
        1. The starting encounter deck should have 20% of these. 
    1. There is a rest deck with different beenfits. 
        1. To start with it will just be: getting more stamina. 
        1. There will be one rest card effect with some rather bit (minmin-maxmin)-(minmax-max) range. 
        1. There will be rolled 5 different rest cards on this. 
        1. Each of those 5 cards have 5 copies in the deck. 
    1. When the encounter start, draw 5 rest cards and pick one. 
        1. That card takes effect straight away and the encounter is won. 

1. Change the vision and roadmap regarding simple implementation of woodcutting: 
    1. It is about hitting a rythm for greater yields. 
    1. Similar durability lost as with herbalism: every card just have a fixed rather small cost. 
        1. Durability is also a second loose condition. 
    1. The rythm is represented in making some pattern. 
    1. So each card have a type: like light chop, heavy chop, medium chop etc. 
        1. like make 5 of them. 
    1. They also have a number between 1-10. 
    1. In fact, each card can have multiple numbers and multiple types: but the initial ones only have 1 of each. 
    1. Tere are no enemy deck in this Diciplin. 
    1. You start with a handsize of 5 and you will be playing 8 cards. Every time you play a card you draw a new card. 
    1. When you have played 8 cards then the best pattern in the cards will reward you "wood tokens". 
    1. Implement many different patterns, get inspired by poker too (where they use 5 cards instead of 8, so it is not an exact match). 
        1. Only the best pattern is used. 
        1. You will likely always get some reward, because there are always some simple patterns. 

# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 

Also, add a section about the copilot instruction file: for more code style kind of instructions suggestions. 
