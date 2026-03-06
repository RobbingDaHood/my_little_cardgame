When the below point states "Roadmap" it means edit the roadmap.md directly.

1. The crafted card is not a new entry in the library: it just adds +1 to the library count for an existing entry. 
1. conclude_crafting_encounter and auto_conclude_craft seems to overlap a lot, could they not be merged? 
1. You cannot abort_crafting_encounter when a craft is ongoing. When ongoing then it only a successfull craft that will make it a win. 
    1. I do not know if it is already like that, if so then do no change. 
1. Crafting cost should be more variable: 
    1. When it is a concrete card it should be static. 
    1. When creting a new concrete card then the crafting cost is decided with some randomness. 
    1. The current "base_cost" is fine, but the distribute it with some ranges on the four gathering tokens. Some can get all the way down to zero. 
        1. At minimum it will require 2 different tokens and at max all of them. 
        1. One token can maximum require 75% of the "base_cost" there are no minimum. 
1. All player cards in all diciplins should have a crafting cost.
1. All enemy deck cards should refer back to enemy card effects. 
    1. This is a preperation for the future scouting step; where we will create new encounters based on the enemy card effects. 
    1. Refactor all enemy cards to also have a enemy card effect. 
    1. These enemy cards and card effects should be part of the diciplin file.
        1. So it is easy to get an overview. 

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
