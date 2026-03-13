When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Why add the "(except GET /actions/possible which returns currently valid player actions with playable card IDs)" to the docs/design/roadmap.md: It is a GET endpoint too, so that is no exception. 
1. In docs/design/roadmap.md it states "The current player actions are: NewGame, EncounterPickEncounter, EncounterPlayCard, EncounterApplyScouting, EncounterAbort.": But there are a lot of other possible actions in the game, like choosing to swap cards during a crafting encounter etc. 
    1. If I am right then update the roadmap and check other places that should be updated too and update them. 
1. Analyse what is run during a github action for a PR, because something there seems to fail. 
    1. Fix what is failing. 
    1. Add to the instruction file to run all the same checks that the github action for PR runs, before comitting. 
1.  So every card effect on every card should refere back to a card effect in the library. 
    1. That library card effect were used to create the card card effect: 
        1. Meaning that the car card effect cannot have values that goes beound the library card effect. 
    1. The Card card effect should have a refference back to what library card effect it came from. 
    1. This means that there likely need to be created a lot more new card effects: In the initial deck. 
        1. It als means that all the card card effects in the intial deck need a reference. 
    1. This is the case both for enemy and player cards. 
    1. Some diciplins do have this setup, but it does not look like all diciplins have this, right? 
1. PossibleAction.action_type should be of the type of PlayerActions
    1. It is fine that it exposes the fields of the enums too: Just to give examples of what can be passed to the action. 
1. PossibleAction.playable_card_ids: Remove that field. 


# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
