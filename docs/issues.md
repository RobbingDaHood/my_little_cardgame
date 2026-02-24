4. **Token map serialization and vision.md**: The compact JSON format ({"Health": 20}) is not documented in vision.md. Since API format is part of the contract, consider documenting it.
    1. Be clear that "Persistant" is the default duration, but that this could be overwritten by any other duration for any type of token.

1. All new cards should be added to the end of the Lilbrary vector and not at the beginning, so all ids are kept stable. 

1. The DrawCards is not a token, but a CardEffect. 

1. The DrawCards effect is a seperate subtype of CardEffects (and there could be other substypes in the future): because this does not manipulate any tokens but just pick up cards on hand. Implement the two subtypes of CardEffect: draw cards and change tokens. 

1. The DrawCards effect draws 2 cards in the initial setup of the game. 

1. Just move the rest of the fields out of Combatant and into CombatState and then get rid of Combatant. 

1. Consider splitting Library.rs up and create more seperate files and folders. 

1. EncounterPhase should not not have a Defence. That is a CombatPhase.

1. Add a new type of automatic test: That tests longer scenarioes. Like all the way from a new game to first combat is won and then start a new scenario etc. They should call the action endpoint and assert relevant get endpoints. The point is to be a very easy "guide" of how to use the interface (besides being a test that some uses cases are reachable). I know it can be tricky to make these decks, because there is some try and error until you find the nessesary steps to get to this case. These tests can never use any test endpoint, only the action endpoint. All these test should be put in one test file: Make sure the test file starts with a good description of what type of tests are in this test file. Some use cases I have in mind for now is: 
    1. Start new game, start encounter, play encounter until the player win, pick new encounter and observe the next combat is started. Also assert that the first game were registered as a win for the player and what tokens the player and enemy had during the battle. 
    1. Then the same case, but just that the enemy won. 

1. Add to the copilot instructions file that I want the test described above updated if there are other use cases added. It should cover all genereal use cases with at least one happy path. 

1. Add to the README that the test described above can be used as a guide on how to play the game. 

# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 
