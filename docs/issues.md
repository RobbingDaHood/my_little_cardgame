1. Seems like the draw_player_cards_of_kind just draws the first card in the drawable and not a random card. It should for each "count" draw a random card in the drawable. 
1. PlayerCardEffect and EnemyCardEffect both have a "lifecykle": But there is not need for it. It is only the TokenType that needs a "lifecykle". So remove that and solely use the life cykle on the Tokentype. 
1. Why is there still an empty tests file at the root of srs? Does not seem like it is needed, so just delete that. 
1. Seems like PlayerData is jsut a random generator wrapper: So just renamed it to that. 
1. More enums: allowed_card_kind should just return a CardKind and then that can be used when comparing types. 
    1. Consider if there are more places where CardKind can be used. 
1. The area endpoint does seem a bit reduntant with the features on the library endpoint. 
1. Consider expanding the Library endpoint filters with "OnHand", so it only returns the cards that has at least one card in hand.
    1. Then other pieces of code could use the same filter logic to get "OnHand" cards of specific CardKinds. 
    1. Consider to also implement "InDeck", "InLIbrary", "InDiscard" etc. 
    1. You can decide what to name the enums, the above is just suggestions. 
1. That means the "area_deck/endpoints.rs" seems redundant and maybe can be deleted? 
1. Looks like the AreaDeck struct is only used in tests now: 
    1. Check if that is strickly needed anymore or it can be deleted. 
    1. If it cannot be deleted then move it to the tests, it should not be in production code. 
1. Seems like ScoutingParams is solely used in the AreaDeck, which is likely either deleted or moved to test. 
    1. If AreaDeck is deleted then consider deleting this too. 
    1. If AreaDeck is moved, then move this too. 
1. The CombatantDef.initial_tokens: The type should be HashMap<Token, u64>, because we do not plan to have any negative token counts. 
    1. Check if it is similar with ther player tokens: if so fix that to u64 too. 
1. ActionPayload seems way to big: If it is solely used for the ActionLog, then it should just be the same as the Player Actions. 
    1. I forgot what the player action enum is called, but currently it only have soemthing like: choose encounter, play card, new game, and some others I forgot. 
1. ActionEntry also seems a bit bloated: 
    1. Is seq used for concurrency purpose? If not then it seems reduntant. 
    1. The payload already decides what "action_type" it is, so "action_type" seems redundant. 
    1. The "actor" is always "the player" because no one else can trigger a player action: So that field seems redundant too. 
    1. I do not know what "the_request" id is, but seems reduntant too. 
    1. The action log is read only so there is no need for a "version", right? 
    1. "timestamp" is "nice to have", so if we remove that too, then there is only the payload: and if the payload is the "Player actions enum" anyway: then ActionEntry can be removed. 
        1. Then just use "Player action enum" directly in the ActionLog. 
1. CombatPhase.allowed_card_kind should use CardKind enum. 
1. CombatState: It is always the players turn or something is being processed, so "player_turn" seems reduntant. 
1. The EncounterState just wraps EncounterPhase so EncounterState seems reduntant. 
1. EncounterPhase: I think Ready is reduntant, because there are no phase just after choosing a encounter and then startig combat: Combat should start ASAP. So we only need NoEncounter, InCombat and Scouting. 
    1. Detail: Renmae "InCombat" to just "Combat", so it aligns with the other enums. 
1. I am still unsure why we need "TokenRegistry", we can just create new tokens directly from the TokenType. 
    1. In the GameState there is a "token_balances" making GameState.registry reduntant. 
    1. So I am quite sure we can just get rid of "TokenRegistry". 
    1. "#[get("/tokens")]" is not needed.
1. You can expand GameState.last_combat_result to a list of combat results: every resovled combat should just be pushed at the end of that Vec. 
    1. Also expose the combat results as a list on an endpoint.
1. GameState.apply_grant: Seems to only be used in tests. 
    1. If it is not needed for anything critical then delete it. 
    1. Else at least move it to test code and remove it from production code. 
1. Same for GameState.apply_consume
1. There are leftover comments like "// tests moved to tests/library_unit.rs" that should just be removed. 

# When done with all of this then update the subbestions-vision-roadmap.md 

It represents suggested changes to vision.md and roadmap.md that would clarify them for the future. 

If I instructed you to do something that you could not read from those two files, then suggest how they can be changed to avoid that in the future. 

Also give me a list of contradictions and areas of improvements. 

Also, add a section about the copilot instruction file: for more code style kind of instructions suggestions. 
