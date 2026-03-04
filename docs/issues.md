When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Fundamenatal change: The rest deck is not on the encounter but is a player deck that lives in the library and have counts like all the others. 
    1. So RestCardEffectTemplate looks like it should be player card effects that can be added to a rest card type.
1. A change: The rest encounter should have a "rest tokens" all starting with just 1-2 at the beginning. 
    1. Everytime a rest card is played then it costs at least one rest token. 
    1. When there are no more rest tokens left or the player stop the encounter then it is marked as won. 
    1. All player card effect for rest can have between 0-2 rest tokens cost: most are in the middle. 
    1. The player cannot loose the rest encounter. 
1. When any CardEffect is a gain effect and have a cost, then the cost have to be a percentage of the gain. 
    1. So start by rolling the gap which is a number range. 
    1. Then roll the gain which is a percentage range based on the cap. 
    1. Then roll the costs which are percentage ranges of the gain (after it is rolled to a number). 
    1. RestCardEffectTemplate Does not do the above yet, it just rolls the costs as numbers too. 
        1. Check if other gain cards have the same error. 

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
