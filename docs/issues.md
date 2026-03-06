When the below point states "Roadmap" it means edit the roadmap.md directly.

1. All the "gain caps" should be very close to the gain because it is pr. gain effect. 
    1. Example: costs: vec![types::GatheringCost {
                    cost_type: types::TokenType::Lumber,
                    amount: 15,
                    light_level_cap: 500,
                }],
    1. So a if a CardEffect ever had two gains of the same type, then they could have individual caps. 
    1. Restructure the CardEffects and concrete Cards to show this. 
1. All tokens that are Encounter specific and live for the duration of an encounter should be placed on the encounter state:
    1. Examples: MiningLightLevel and MiningYield 
    1. Look for other examples and move them all to EncounterState tokens. 
    1. This makes it trivial to cleanup a enconter and identify encpunter specifi tokens. 
1. MiningPower is not accumulated, but is used solely during a play of a card, so there are no need to have tokens for that. 
1. Consider renaming GatheringCost to Tokens and then cost_type to type. 
    1. GatheringCost is used both in gains and costs so the name is not optimal. 
    1. After the rename consider if Tokens could be used more places: It should be usable every place where we have an amount of tokens. 
        1. Replace all those cases. 
1. It important that playing any card effect with a cap never reduces the current accumulated amount of tokens. 
    1. Look like the current implmentation of MiningLightLevel does the opposite: If the light level is above the cap it will be reduced to the cap. 
    1. The intend of any cap is to reduce the gain if it is above the cap, not to reduce the accumulated value. 
        1. So a more powerfull card effects in the form of gains could be limited by its cap. 
1. If the player dies then:
    1. He looses all gathering tokens and starts with the initial health and stamina. 
        1. Any non gathering tokens are kept. 
        1. All cards are kept. 
    1. The is a "player deaths" token that accumulates by every player death. 
1. Remove "OreHealth" and "ore_tokens" if it is not used, we do not need to respect backwards compatibility. 
1. Scenario tests must discover card IDs dynamically via API queries (e.g., `/library/cards?card_kind=Mining&location=Hand`) rather than hardcoding index-based IDs, since card registration order determines IDs and any card addition/removal shifts all subsequent IDs.
1. Card effects should express all behavior through token types in cost/gain vectors rather than custom-named fields. Resolution logic interprets the token type to determine behavior.
    1. Refactor where needed. Do not respect backwards compatibility. 
1. Every diciplin that can conclude without an abort should be using EncounterConcludeEncounter
    1. Mining already does this. 
    1. All the other diciplins except combat should be able to do this too: but only if they have accumulated some amount of reward. 
        1. If there are no reward accumulated yet, then the player can only abort and so mark the encounter as lost. 
        1. Using EncounterConcludeEncounter will mark the encounter as success if there is a reward accumulated. 
1. Ensure that all tests are isolated and never interferes with each other: So they could even be run in parallel. 

1. Roadmap: Renumber "Crafting encounters and discipline"-step to be the last 9.x step. 
1. Roadmap: Remove "11.5) Gathering balance pass"
1. Roadmap: Move "Implement Trading and Merchants (MerchantOffers + Barter workflow)" to an "ideas" section later: Add a note that we both need to know if it is needed and what the unique gameplay of the barter game should be. 
1. Roadmap: Remove "Add persistent player progression, library-driven deck-building, and upgrades"
1. Roadmap: Remove "Add resource management, camp mechanics, and short-term tokens"
1. Roadmap: Remove "Implement varied enemy AI, conditional card effects, and targeting"
1. Roadmap: Keep "Introduce persistent world/meta-progression and milestone systems" But just rename it "milestone encounters" 
    1. Also add that there should be a "Milestone encounter" for each interesting CardEffect (need to make a list later), when the "milestone" is beaten then a more powerfill version will be created that will reward the next "tier" of that card effect. Some of the "milestones" will reward tokens like "max hand size" etc: usually these would require to beat some of the other "milestones" first to get a "token key" from them. 
1. Roadmap: Add a step after milstones: Move all configuration out in json files that are loaded durng compilation. 
    1. Initial library (with all the cards in there), tokens and other configurations. 
    1. There should be a new folder at the root called configurations. 
        1. Keep a substructre pr. diciplin and a general for the rest. So it is easy to get an overview of each diciplin.
    1. The impact of this folder should be in the compiled code: So a compiled game cannot change these configurations, but we can change these values before compiling. 
1. Roadmap: Add a step after "UX polish, documentation, tools for designers, and release" 
    1. Balancing setup: 
        1. Define balancing goals. 
        1. Setup a mutating runner, that tries out different strategies and document if they are all somewhat good at reaching a specific goal and that there are many ways to get to the goal. 
            1. Maybe the scope should be limited to balancing each of the diciplins: We want multiple possible strategies for winning in each of the diciplins, that all should be viable and interesting. 
            1. Also define expected outcome of each encounter pr. tier and the expected fail/success rate. 
        1. Trigger the runners and get some data. 
        1. Analyze data and make adjustments. 
1. Roadmap: Renumber the points in the plan so there are no gaps. 


# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
