use crate::library::types::{
    self, CardCounts, CardKind, EncounterKind, EncounterOutcome, EncounterState, RestEncounterState,
};
use crate::library::{GameState, Library};

pub(crate) fn register_rest_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    // Three rest card effect templates:
    // 1. Great Stamina recovery — costs fish + herbs
    let stamina_template = types::RestCardEffectTemplate {
        recoveries: vec![types::RestRecoveryRange {
            token_type: types::TokenType::Stamina,
            cap_min: 400,
            cap_max: 600,
            gain_min_percent: 80,
            gain_max_percent: 100,
        }],
        cost_ranges: vec![
            types::RestCostRange {
                cost_type: types::TokenType::Fish,
                min_amount: 50,
                max_amount: 150,
            },
            types::RestCostRange {
                cost_type: types::TokenType::Plant,
                min_amount: 50,
                max_amount: 150,
            },
        ],
    };

    // 2. Health recovery — costs fish + herbs
    let health_template = types::RestCardEffectTemplate {
        recoveries: vec![types::RestRecoveryRange {
            token_type: types::TokenType::Health,
            cap_min: 300,
            cap_max: 500,
            gain_min_percent: 80,
            gain_max_percent: 100,
        }],
        cost_ranges: vec![
            types::RestCostRange {
                cost_type: types::TokenType::Fish,
                min_amount: 80,
                max_amount: 200,
            },
            types::RestCostRange {
                cost_type: types::TokenType::Plant,
                min_amount: 80,
                max_amount: 200,
            },
        ],
    };

    // 3. Mixed Stamina + Health — less total than specialized
    let mixed_template = types::RestCardEffectTemplate {
        recoveries: vec![
            types::RestRecoveryRange {
                token_type: types::TokenType::Stamina,
                cap_min: 200,
                cap_max: 350,
                gain_min_percent: 70,
                gain_max_percent: 90,
            },
            types::RestRecoveryRange {
                token_type: types::TokenType::Health,
                cap_min: 150,
                cap_max: 300,
                gain_min_percent: 70,
                gain_max_percent: 90,
            },
        ],
        cost_ranges: vec![
            types::RestCostRange {
                cost_type: types::TokenType::Fish,
                min_amount: 30,
                max_amount: 120,
            },
            types::RestCostRange {
                cost_type: types::TokenType::Plant,
                min_amount: 30,
                max_amount: 120,
            },
        ],
    };

    // Roll 5 concrete rest cards from the 3 templates.
    // Distribution: 2 stamina, 2 health, 1 mixed. Most have costs.
    // The rest deck should be mainly cards with cost and only a few without.
    let templates = [
        &stamina_template,
        &stamina_template,
        &health_template,
        &health_template,
        &mixed_template,
    ];

    let mut rest_cards = Vec::new();
    for (i, template) in templates.iter().enumerate() {
        let recoveries = template
            .recoveries
            .iter()
            .map(|r| {
                let rolled_cap = crate::library::game_state::roll_range(rng, r.cap_min, r.cap_max);
                let rolled_gain = crate::library::game_state::roll_range_u32(
                    rng,
                    r.gain_min_percent,
                    r.gain_max_percent,
                );
                types::ConcreteRestRecovery {
                    token_type: r.token_type.clone(),
                    rolled_value: rolled_cap * rolled_gain as i64 / 100,
                    rolled_cap,
                    rolled_gain_percent: rolled_gain,
                }
            })
            .collect();

        // Card at index 4 (the mixed card) is the cost-free one
        let costs = if i == 4 {
            vec![]
        } else {
            template
                .cost_ranges
                .iter()
                .map(|c| types::GatheringCost {
                    cost_type: c.cost_type.clone(),
                    amount: crate::library::game_state::roll_range(rng, c.min_amount, c.max_amount),
                })
                .collect()
        };

        rest_cards.push(types::RestCard {
            recoveries,
            costs,
            counts: types::DeckCounts {
                deck: 5,
                hand: 0,
                discard: 0,
            },
        });
    }

    // Register rest encounter card (~20% of encounter deck = hand: 4)
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Rest {
                rest_def: types::RestDef {
                    rest_deck: rest_cards,
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 4,
            discard: 0,
        },
    );
}

impl GameState {
    /// Initialize a rest encounter from a Library Encounter card.
    /// Draws 5 rest cards from the rest deck and presents them as choices.
    pub fn start_rest_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let rest_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Rest { rest_def },
            } => rest_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a rest encounter",
                    encounter_card_id
                ))
            }
        };

        let mut rest_deck = rest_def.rest_deck;
        // Shuffle the rest deck
        crate::library::game_state::deck_shuffle_hand(rng, &mut rest_deck);

        // Draw 5 rest cards from deck to hand
        for _ in 0..5 {
            crate::library::game_state::deck_draw_random(rng, &mut rest_deck);
        }

        let state = RestEncounterState {
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            rest_hand: rest_deck,
        };
        self.current_encounter = Some(EncounterState::Rest(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Player picks a rest card by index into the rest_hand.
    /// Applies recovery effects (respecting caps), deducts costs, marks encounter as won.
    pub fn resolve_rest_card_choice(&mut self, rest_card_index: usize) -> Result<(), String> {
        let rest_card = {
            let rest_state = match &self.current_encounter {
                Some(EncounterState::Rest(r)) => r,
                _ => return Err("No active rest encounter".to_string()),
            };

            // Find the card in hand at the given index
            let hand_cards: Vec<&types::RestCard> = rest_state
                .rest_hand
                .iter()
                .filter(|c| c.counts.hand > 0)
                .collect();

            if rest_card_index >= hand_cards.len() {
                return Err(format!(
                    "Rest card index {} out of range (hand has {} cards)",
                    rest_card_index,
                    hand_cards.len()
                ));
            }

            hand_cards[rest_card_index].clone()
        };

        // Check and deduct costs (herbs and fish tokens)
        GameState::check_and_deduct_gathering_costs(&rest_card.costs, &mut self.token_balances)?;

        // Apply recovery effects (respecting caps)
        for recovery in &rest_card.recoveries {
            let entry = types::token_entry_by_type(&mut self.token_balances, &recovery.token_type);
            *entry += recovery.rolled_value;
            // Cap the token at rolled_cap
            if *entry > recovery.rolled_cap {
                *entry = recovery.rolled_cap;
            }
        }

        // Mark encounter as won
        self.last_encounter_result = Some(EncounterOutcome::PlayerWon);
        self.encounter_results.push(EncounterOutcome::PlayerWon);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;

        Ok(())
    }
}
