1. The draw card effects should define a list of "decks" that get X amount of cards drawn. 
    1. Example: One draw card effect could draw 1 card in the attack deck, 1 card in the defence deck and 2 cards in the ressource deck. 
    1. So the draw card effect always have a map of "CardType" to counter. Representing the collections of "decks" that are affected and how many cards there is drawn for each. 
    1. To be clear it all interacts with the library as it is currently implmented and only the Draw CardEffect and the code that executes the card effect need to change. 
        1. This is because the attack deck/hand is defined as all the cards in the library with the attack card type: etc. 
    1. I want the initial created cards at a new game to each to give exacly: 1 attack, 1 defence and 2 ressource draw cards. 
1. The DrawCard effect does now seem to be implemented in the apply_card_effect. Implent this. 
    1. The DrawCard effect should "draw a card" for the relevant deck. 
    1. If an enemy playes the card then the enmy will draw a card. 
    1. If the palyer playes it they can draw a card. 
    1. It respect the library way of counting. 
    1. Add that both player and enemy draws some cards to the scenario tests: it is fine if that just get added to at least one existing test. 
    1. When the player draws cards, then it is random cards from the relevant "deck" and on hand. 
    1. If there are not any more cards left in the relevant deck, then move all cards from "discard" to "deck" and then draw. 
1. CardDef.card_type should be an enum.
1. CardEffect.kind should be defined by whatever "card_effect_id" is pointing at. 
1. CardEffect.card_effect_id should not be optional but is mandatory.
1. CardEffect.lifecycle should just be deletd: If the card effect that is reffered too manipulates with tokens, then their lifecycle is defined there. 
1. Seems like this means that the CardEffect struct is not needed anymore? if se delete it and just use the card CardEffect. 
1. The deck struct seems unused, if it is then delete it. 
1. Make a bigger more thorough analasys of the codebase and refacor:
    1. Get rid of legacy code: do not consider breaking changes.
    1. Get rid of dead code. 
    1. Optimize the code and simplify the implementations. 
    1. Cannot loose any functionality on the public /action POST endpoint: But else I think the rest can be changed. 

# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 
