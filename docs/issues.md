When the below point states "Roadmap" it means edit the roadmap.md directly.

1. The choosen "Disciplin" when choosing a ressearch project decides the type of the card that is added to the library if the ressearch is concluded. 
    1. So the ResearchProject struct does not need to mention discipline, will be part of the chosen_card.
        1. Does the ResearchProject need tier_count? Seems like total_cost is sufficent for all the logic.
1. Rename the "tag" on card effects to "valid_diciplin_types". 
    1. The field is a list of "types::Discipline"s
1. Every diciplin should process the insight card effect. 
1. Remove all the mentions of the ressearch scenario is not in hand from vision and roadmap.    
    1. That is irrelevant and subject to the seed used. 
1. Add a get endpoint that exposes a list of possible actions for the player to play currently.
1. BTW: The enemy card effects also have tags/valid_diciplin_types, they will be used later for the scouting step. 
    1. They should also use the diciplin enum. 
1. Change the github action to only require 80 percent test coverage instead of 85. 
1. There are still places where the cap is limiting the amount of tokens on the player tokens and not solely the capping the gains from a card effect. 
    1. All caps limit the gain and never limit the total amount on the player tokens. 
    1. Find all references to caps in roadmap and vision and make sure they state the same. 
1. Consider the current automatic test suite and if any test overlaps with an existing scenario_test the remove that test and keep the scenario_test. 
    1. If this means we can remove some test endpoints, then do that. 
1. Durability card effects are generalized — the discipline context determines which durability pool is affected
    1. Implement this. 
1. Clarify the vision: The "per-dicipline insight tokens means": 
    1. That there is one pool of insight tokens pr. diciplin. 
    1. When a insight card effect is played, then consider what encounter it is played in and add that insight to the correct insight pool. 
    1. Make a diciplin specific insight token for each diciplin: a bit similar to durabilit, but just for every dciplin. 
1. Remove all mentions about a generalized durability token setup, there should keep being per-diciplin durability tokens. 
    1. Do consider if some card effects could be generalized: 
        1. So the Card effect just rewards or removes durability. 
        1. The "valid_diciplin_types" are then every diciplin. 
        1. When the card effect is triggered then based on the encounter it will be added to the correct diciplin pool of tokens. 
            1. The rest encounter card effect have to mention a specific diciplin and cannot be generalized. 
1. Add more card effects that costs stamina and health. 
    1. The benefit of a healt cost card should be great.
    1. The benefit of a stamina cost card should be greater than no cost cards and smaller than health cost cards. 
    1. Every diciplin should have one starting card with such a card. 
1. Update vision-roadmap-suggestions.md with all changes implemnted after implementing all suggestions in this document. 
    1. Remove all parts that are not relevant anymore. 
1. Migrate encounter resolution logic to use ConcreteEffect-based effects instead of hardcoded fields (damages, value, increases, characteristics)
    1. Consider to remove the legacy fields (`damages`, `value`, `increases`).

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
