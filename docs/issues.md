4. **Token map serialization and vision.md**: The compact JSON format ({"Health": 20}) is not documented in vision.md. Since API format is part of the contract, consider documenting it.

1. All new cards should be added to the end of the Lilbrary vector and not at the beginning, so all ids are kept stable. 

1. The DrawCards is not a token, but a CardEffect. 

1. The DrawCards effect is a seperate subtype of CardEffects (and there could be other substypes in the future): because this does not manipulate any tokens but just pick up cards on hand. Implement the two subtypes of CardEffect: draw cards and change tokens. 

1. The DrawCards effect draws 2 cards in the initial setup of the game. 

1. Just move the rest of the fields out of Combatant and into CombatState and then get rid of Combatant. 

1. Consider splitting Library.rs up and create more seperate files and folders. 

1. EncounterPhase should not not have a Defence. That is a CombatPhase.


# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 
