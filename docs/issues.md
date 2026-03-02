When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Split the ChangeTokens CardEffectKind into two: One that is a GainTokens and another that is LooseTokens 
    1. The GainTokens keep the cap fields. 
        1. They also start with the cap and then calcualate a gain. 
        1. They can still have a cost as a percentage of the gain. 
        1. They can never gain somet type of token that is also a cost: on the same CardEffect
    1. The LooseTokens do not need the cap fields. 
        1. They calculate the value and then afterward the cost. 
    1. This way the enums are not bloated with fields that are usually empty or zero. 
1. ~~The stamina_grant fields should be part of a "gains" list of vector (token, amount)s.~~ (Done — replaced with `gains: Vec<GatheringCost>`) 
1. All the new "modify_"-fields should also be new tokens or manipulate with a token. 
    1. The rewards on an encunter is a vec of (token, amount): So increase of rewards on a CardEffect is just a "gain" with the correct token. 
1. Durability lost on all the card effects should be part of the "cossts" as one of the tokens that will be paid. 
1. If in doubt the make it a token: Then it is either on the player, the encounter or on a card effects as either "costs" or "gains"; There should not be much else in form of fields. 
1. The "target_characteristics" in herbalism should be of the HerbalismMatchMode type and then each type wraps the relevant array or values. 
    1. So the OR and AND wraps a vector. 
    1. MostCommon and LeastCommon wraps a limit and a vector of types. 
1. The initial max handsizes are 5 not 10. 
1. If no card effect costs on a card can be paid then the card play errors and the player can choose another card. 
1. THe "all_combat_hand_cards_unpayable" logic is not only relevant for combat decks, they are relevant for every single player deck that can be used in encounters. 
    1. If they player cannot play any cards because they cannot pay at least one effect on the card, then the player looses the encounter. 
1. Split the "game_state.rs" file logic into a file pr. diciplin: group all code that relates to one diciplin in that file. 
    1. consider if it makes sense to have a subfolder for diciplins. 
    1. consider refactoring the code to better support this divide of the code. 
    1. General methods like paying costs etc. can stay in game_state or maybe be exportet to some helper file. 

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
