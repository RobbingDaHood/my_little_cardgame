# Summary

The following is a list of issues in the current implementation. 

When I mention integration tests, then I mean: Spinning up the server and call endpoints to achieve what not to be tested. 

# We can add cards in a cards POST endpoint. 

It is fine to have this for now for testing purposes, please add to the documentation of this endpoint that it is solely for testing purposes and at a later point in the roadmap it needs to be removed. Move the POST cards endpoint to "/tests/cards/" to make it very clear it is a testing endpoint. 

Maybe it can be removed as early as when we have implemented the ressearch flow: so we now can add cards. 

Because of the reproduceable nature of this project, then it is fine to setup somewhat complex integration tests that plays the game to a point where we add the cards we are interested in. Because everything is in memory-then it should not be that slow. 

# There still seems to be a decks endpoint 

That is a leftover from before the vision and roadmap, from before we started refactoring. 

I do not think it is nessesary anymore. Because:
1. The library should have all the cards.
1. The player only have one deck of each type. 
1. So when the library states that X cards are "on hand" then we know what deck they are in. 
1. So a filter on the library get endpoint where it is possible to filter by card type and "is at lean one on hand" would be a replacement for the decks endpoints. 

I want this to be changed now. Implement the remaing refactor so the endpoints are clear and make sure all tests are updated too. Remember that we favour testing through the endpoint in integration tests. 

I assume a lot of complexity can be remvoed from the code: because now it i not possible to add cards to the wrong type, create decks etc. 

## Side point: Testing endpoints

If this means that we currently cannot test some functionality, then implement some nessesary endpoints under "/tests" to achieve coverage. 

# If the combat is not initialized yet then return 404 and no null

That makes for a cleaner interface. 

# Remove all descriptions and names from the game. 

I am quite sure that is part of the vision.md

We will not use time on naming anything like "forest_area" or "Ancient Forest". The only thing that does have names are enums, classes, variables etc. all the data. 

It is left up to the imagination of the user what the different entities are. 

# AreaDeck represents wherever the player is right now

So there will not be multiple area decks, there will be one area deck that represents where the player is now. Then that deck will loose and add new encounters as the player progresses. 

Make sure the code reflects this. 

As an example (and I did not check if that is already implemented like this): All cards in the "AreaDeck" is in the library like all the other cards with the counts like all the other cards: with an area type. No need to represent them otherwise anywhere. 

# ScoutingParams

I am quite sure that is not part of the vision. I can also see the vision is a bit unclear. So here is some clear statements about scouting: 
1. It is a post-encounter step and not an encounter on its own. 
1. Scouting gives no bias and no affinity.
    1. I can see this is written here and there in the vison but that is wrong. 
1. The scouting post-encounter step is all about building the next encounter, as stated in the "affix generation" under scouting in the vision. 
1. There are in total 3 scouting related tokens
    1. Foresight: Deciding the max handsize of the "area dek"
        1. So this token is not directly related to the scouting post-encounter step
    1. Scouting candidate pool size: Controls how many affix cards are picked for the new card during the scouting post-encounter step. 
        1. This is part of roadmap step 7, so maybe not relevant yet. 
    1. Scouting canditate pick size: How many affixes can be picked for the new card during the scouting post-encounter step. 
        1. This is part of roadmap step 7, so maybe not relevant yet. 

Quite a lot of this is likely implemented in future steps of the roadmap, but I see stuff like ScoutingParams as a contradiction of the above: So here are the options:
1. Either replace it with something that is closer to the vision
1. Move it to some test endpoint until it can be replaced in future steps. Only pick this option if: 
    1. It is critical for tests and test coverage. 
    1. And that it would be very complex to move closer to the vison. 
    1. So likely you should just replace it. 

# Initiaze comabe should be an /action and not a POST comabat endpoint

All mutations should go through the action endpoint. 

Only exceptions are endpoints mved to "/test" for testing purpose. 

So if the POST "/combat" is strictly needed for tests then move it to "/tests/combat". 

I am quite sure you can just use "/action" now. 

# Tokens are it own endpoint and not under the library

So it is is GET "/tokens" and not "/library/tokens". 

# Add 80% test coverage in the form of "integration tests"

An by integration tests i mean tests that spins up the server and calls the endpoints until some use case have been asserted. 

This is because these tests are very easy to review and see how the interface "works" and in the current setup it should not be too expensive to execute. 

# Clarify vision
When done with all the above then analyze the vision and roadmap and suggest (do not change) imporvements. Espetially if some of the above fixes were not clear from the vision, then i want to add parts to the vision so it alligns. 

Make both a vision_suggest.md document with all the suggestions and a final_vision.md where all the suggestions have been applied to a copy of the vision.md. None of these files may be places in docs/design. 
