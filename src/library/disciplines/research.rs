use crate::library::types::{
    self, CardCounts, CardEffectKind, CardKind, Discipline, EncounterKind, EncounterOutcome,
    EncounterState, ResearchCandidate, ResearchEncounterState, ResearchRoundResult, ResearchSymbol,
};
use crate::library::{GameState, Library};
use rand::RngCore;

use crate::library::game_state::roll_concrete_effect;

const DEFAULT_TARGET_SIZE: u32 = 3;
const DEFAULT_POSITION_MATCH_YIELD: i64 = 100;
const DEFAULT_TYPE_MATCH_YIELD: i64 = 10;
const DEFAULT_BASE_INSIGHT_COST: i64 = 5;

pub(crate) fn register_research_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    // Encounter card
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Research {
                research_def: types::ResearchDef {
                    target_size: DEFAULT_TARGET_SIZE,
                    position_match_yield: DEFAULT_POSITION_MATCH_YIELD,
                    type_match_yield: DEFAULT_TYPE_MATCH_YIELD,
                    base_insight_cost: DEFAULT_BASE_INSIGHT_COST,
                },
            },
        },
        CardCounts {
            library: 0,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![Discipline::Research],
    );

    // ---- Research PlayerCardEffect templates ----

    // Basic probe: single symbol, no cost
    let all_symbols = ResearchSymbol::all();
    let mut basic_effect_ids = Vec::with_capacity(all_symbols.len());
    for symbol in all_symbols {
        let id = lib.add_card(
            CardKind::PlayerCardEffect {
                kind: CardEffectKind::ResearchProbe {
                    symbols: vec![symbol.clone()],
                    costs: vec![],
                },
            },
            CardCounts {
                library: 1,
                deck: 0,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![Discipline::Research],
        );
        basic_effect_ids.push(id);
    }

    // Premium probe: two symbols, costs Stamina
    let premium_pairs: Vec<(ResearchSymbol, ResearchSymbol)> = vec![
        (ResearchSymbol::Alpha, ResearchSymbol::Beta),
        (ResearchSymbol::Gamma, ResearchSymbol::Delta),
        (ResearchSymbol::Epsilon, ResearchSymbol::Zeta),
    ];
    let mut premium_effect_ids = Vec::with_capacity(premium_pairs.len());
    for (s1, s2) in &premium_pairs {
        let id = lib.add_card(
            CardKind::PlayerCardEffect {
                kind: CardEffectKind::ResearchProbe {
                    symbols: vec![s1.clone(), s2.clone()],
                    costs: vec![types::CardEffectCost {
                        token_type: types::TokenType::Stamina,
                        min_percent: 5,
                        max_percent: 10,
                        is_absolute: true,
                    }],
                },
            },
            CardCounts {
                library: 1,
                deck: 0,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![Discipline::Research],
        );
        premium_effect_ids.push(id);
    }

    // Premium probe: three symbols, costs Health
    let triple_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::ResearchProbe {
                symbols: vec![
                    ResearchSymbol::Alpha,
                    ResearchSymbol::Gamma,
                    ResearchSymbol::Epsilon,
                ],
                costs: vec![types::CardEffectCost {
                    token_type: types::TokenType::Health,
                    min_percent: 3,
                    max_percent: 5,
                    is_absolute: true,
                }],
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![Discipline::Research],
    );

    let triple2_id = lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::ResearchProbe {
                symbols: vec![
                    ResearchSymbol::Beta,
                    ResearchSymbol::Delta,
                    ResearchSymbol::Zeta,
                ],
                costs: vec![types::CardEffectCost {
                    token_type: types::TokenType::Health,
                    min_percent: 3,
                    max_percent: 5,
                    is_absolute: true,
                }],
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![Discipline::Research],
    );

    // ---- Concrete Research cards ----

    // Basic cards: 3 copies each, single symbol
    for &effect_id in &basic_effect_ids {
        let concrete = roll_concrete_effect(rng, effect_id, lib);
        lib.add_card(
            CardKind::Research {
                effects: vec![concrete],
            },
            CardCounts {
                library: 0,
                deck: 3,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![Discipline::Research],
        );
    }

    // Premium dual-symbol cards: 2 copies each
    for &effect_id in &premium_effect_ids {
        let concrete = roll_concrete_effect(rng, effect_id, lib);
        lib.add_card(
            CardKind::Research {
                effects: vec![concrete],
            },
            CardCounts {
                library: 0,
                deck: 2,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![Discipline::Research],
        );
    }

    // Premium triple-symbol cards: 1 copy each
    for effect_id in [triple_id, triple2_id] {
        let concrete = roll_concrete_effect(rng, effect_id, lib);
        lib.add_card(
            CardKind::Research {
                effects: vec![concrete],
            },
            CardCounts {
                library: 0,
                deck: 1,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![Discipline::Research],
        );
    }
}

/// Extract the ResearchSymbols from a Research card's effects.
fn card_symbols(card: &types::LibraryCard, lib: &Library) -> Vec<ResearchSymbol> {
    let effects = match &card.kind {
        CardKind::Research { effects } => effects,
        _ => return vec![],
    };
    let mut symbols = Vec::new();
    for effect in effects {
        if let Some(ref_card) = lib.get(effect.effect_id) {
            if let CardKind::PlayerCardEffect {
                kind: CardEffectKind::ResearchProbe { symbols: s, .. },
            } = &ref_card.kind
            {
                symbols.extend(s.iter().cloned());
            }
        }
    }
    symbols
}

/// Find the 1:1 assignment of cards to hidden slots maximizing total yield.
/// Returns per-card yield values.
fn optimal_matching(
    card_symbols_list: &[Vec<ResearchSymbol>],
    hidden_types: &[ResearchSymbol],
    position_yield: i64,
    type_yield: i64,
) -> Vec<i64> {
    let n = hidden_types.len();
    assert_eq!(card_symbols_list.len(), n);

    let mut best_total = 0i64;
    let mut best_per_card = vec![0i64; n];

    // Generate all permutations of slot assignments for cards
    let perms = permutations(n);
    for perm in &perms {
        let mut per_card = vec![0i64; n];
        let mut total = 0i64;
        for (card_idx, &slot_idx) in perm.iter().enumerate() {
            let card_syms = &card_symbols_list[card_idx];
            let hidden_sym = &hidden_types[slot_idx];
            if card_syms.contains(hidden_sym) {
                let yield_val = if card_idx == slot_idx {
                    position_yield
                } else {
                    type_yield
                };
                per_card[card_idx] = yield_val;
                total += yield_val;
            }
        }
        if total > best_total {
            best_total = total;
            best_per_card = per_card;
        }
    }

    best_per_card
}

fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    let mut current: Vec<usize> = (0..n).collect();
    permute(&mut current, 0, &mut result);
    result
}

fn permute(arr: &mut Vec<usize>, start: usize, result: &mut Vec<Vec<usize>>) {
    if start == arr.len() {
        result.push(arr.clone());
        return;
    }
    for i in start..arr.len() {
        arr.swap(start, i);
        permute(arr, start + 1, result);
        arr.swap(start, i);
    }
}

impl GameState {
    pub fn start_research_encounter(&mut self, encounter_card_id: usize) -> Result<(), String> {
        if self.current_encounter.is_some() {
            return Err("Already in an encounter".to_string());
        }

        let card = self
            .library
            .get(encounter_card_id)
            .ok_or("Encounter card not found")?
            .clone();
        match &card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Research { .. },
            } => {}
            _ => return Err("Card is not a research encounter".to_string()),
        }

        self.current_encounter = Some(EncounterState::Research(ResearchEncounterState {
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            candidates: None,
            hidden_types: None,
            accumulated_yield: 0,
            rounds_played: 0,
            round_history: vec![],
            experiment_active: false,
        }));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    pub fn research_choose_project(
        &mut self,
        discipline: Discipline,
        tier_count: u32,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Research(r)) if r.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active research encounter".to_string()),
        }

        if tier_count == 0 {
            return Err("Tier count must be at least 1".to_string());
        }

        // Cost: 10 * 2^(tier_count - 1)
        let insight_cost = 10_i64 * (1i64 << (tier_count - 1));

        let insight_token = types::TokenType::insight_for_discipline(&discipline);
        let insight_key = types::Token::persistent(insight_token);
        let balance = self.token_balances.get(&insight_key).copied().unwrap_or(0);
        if balance < insight_cost {
            return Err(format!(
                "Insufficient Insight: need {} but have {}",
                insight_cost, balance
            ));
        }

        let matching_effects = self.library.card_effects_for_discipline(&discipline);
        if matching_effects.is_empty() {
            return Err(format!(
                "No card effects available for discipline {:?}",
                discipline
            ));
        }

        *self.token_balances.entry(insight_key).or_insert(0) -= insight_cost;

        let effect_ids: Vec<usize> = matching_effects.iter().map(|(id, _)| *id).collect();

        let mut candidates = Vec::with_capacity(3);
        for _ in 0..3 {
            let mut effects = Vec::with_capacity(tier_count as usize);
            for _ in 0..tier_count {
                let idx = (rng.next_u64() as usize) % effect_ids.len();
                let effect_id = effect_ids[idx];
                let concrete = roll_concrete_effect(rng, effect_id, &self.library);
                effects.push(concrete);
            }
            candidates.push(ResearchCandidate {
                discipline: discipline.clone(),
                effects,
                tier_count,
            });
        }

        if let Some(EncounterState::Research(r)) = &mut self.current_encounter {
            r.candidates = Some(candidates);
        }

        Ok(())
    }

    pub fn research_select_candidate(&mut self, candidate_index: usize) -> Result<(), String> {
        let candidates = match &self.current_encounter {
            Some(EncounterState::Research(r))
                if r.outcome == EncounterOutcome::Undecided && r.candidates.is_some() =>
            {
                r.candidates.as_ref().ok_or("No candidates generated")?
            }
            _ => return Err("No active research encounter with candidates".to_string()),
        };

        if candidate_index >= candidates.len() {
            return Err(format!(
                "Invalid candidate index {}, must be 0-{}",
                candidate_index,
                candidates.len() - 1
            ));
        }

        let chosen = candidates[candidate_index].clone();
        let tier_count = chosen.tier_count;
        // Total research cost: 20 * 2^(tier_count - 1)
        let total_cost = 20_i64 * (1i64 << (tier_count - 1));

        self.current_research = Some(types::ResearchProject {
            chosen_card: chosen,
            progress: 0,
            total_cost,
        });

        if let Some(EncounterState::Research(r)) = &mut self.current_encounter {
            r.candidates = None;
        }

        Ok(())
    }

    pub fn research_progress(
        &mut self,
        amount: i64,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Research(r)) if r.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active research encounter".to_string()),
        }

        let project = self
            .current_research
            .as_ref()
            .ok_or("No active research project")?;

        if amount <= 0 {
            return Err("Progress amount must be positive".to_string());
        }

        let remaining_cost = project.total_cost - project.progress;
        // Cap at 33% of total cost
        let max_per_action = (project.total_cost + 2) / 3;
        let insight_token =
            types::TokenType::insight_for_discipline(&project.chosen_card.discipline);
        let insight_key = types::Token::persistent(insight_token);
        let available = self.token_balances.get(&insight_key).copied().unwrap_or(0);

        let actual = amount
            .min(max_per_action)
            .min(available)
            .min(remaining_cost);
        if actual <= 0 {
            return Err(
                "Cannot make progress: insufficient Insight or already complete".to_string(),
            );
        }

        *self.token_balances.entry(insight_key).or_insert(0) -= actual;

        let project = self
            .current_research
            .as_mut()
            .ok_or("No active research project")?;
        project.progress += actual;

        if project.progress >= project.total_cost {
            let finished = self
                .current_research
                .take()
                .ok_or("Research project disappeared")?;

            self.library.add_card(
                CardKind::Attack {
                    effects: finished.chosen_card.effects.clone(),
                },
                CardCounts {
                    library: 0,
                    deck: 0,
                    hand: 0,
                    discard: 0,
                },
                rng,
                vec![finished.chosen_card.discipline.clone()],
            );
        }

        Ok(())
    }

    /// Begin the hidden-multiplier experiment phase: generate hidden types and draw research cards.
    pub fn research_begin_experiment(
        &mut self,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let research_def = match &self.current_encounter {
            Some(EncounterState::Research(r)) if r.outcome == EncounterOutcome::Undecided => {
                if r.experiment_active {
                    return Err("Experiment already active".to_string());
                }
                // Get the research def from the encounter card
                let card = self
                    .library
                    .get(r.encounter_card_id)
                    .ok_or("Encounter card not found")?;
                match &card.kind {
                    CardKind::Encounter {
                        encounter_kind:
                            EncounterKind::Research {
                                research_def: def, ..
                            },
                    } => def.clone(),
                    _ => return Err("Not a research encounter card".to_string()),
                }
            }
            _ => return Err("No active research encounter".to_string()),
        };

        // Generate hidden types
        let all_symbols = ResearchSymbol::all();
        let mut hidden = Vec::with_capacity(research_def.target_size as usize);
        for _ in 0..research_def.target_size {
            let idx = (rng.next_u64() as usize) % all_symbols.len();
            hidden.push(all_symbols[idx].clone());
        }

        // Draw research cards from deck to hand (up to 7)
        let max_hand = 7usize;
        let mut drawn = 0;
        for i in 0..self.library.cards.len() {
            if drawn >= max_hand {
                break;
            }
            if matches!(self.library.cards[i].kind, CardKind::Research { .. })
                && self.library.cards[i].counts.deck > 0
            {
                self.library.cards[i].counts.deck -= 1;
                self.library.cards[i].counts.hand += 1;
                drawn += 1;
            }
        }

        if let Some(EncounterState::Research(r)) = &mut self.current_encounter {
            r.hidden_types = Some(hidden);
            r.experiment_active = true;
            r.accumulated_yield = 0;
            r.rounds_played = 0;
            r.round_history.clear();
        }

        Ok(())
    }

    /// Play a hand of research cards against the hidden multipliers.
    pub fn research_play_hand(
        &mut self,
        card_ids: Vec<usize>,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        // Validate encounter state
        let (hidden_types, research_def, rounds_played) = match &self.current_encounter {
            Some(EncounterState::Research(r))
                if r.outcome == EncounterOutcome::Undecided && r.experiment_active =>
            {
                let hidden = r
                    .hidden_types
                    .as_ref()
                    .ok_or("No hidden types generated")?
                    .clone();
                let card = self
                    .library
                    .get(r.encounter_card_id)
                    .ok_or("Encounter card not found")?;
                let def = match &card.kind {
                    CardKind::Encounter {
                        encounter_kind: EncounterKind::Research { research_def, .. },
                    } => research_def.clone(),
                    _ => return Err("Not a research encounter card".to_string()),
                };
                (hidden, def, r.rounds_played)
            }
            _ => return Err("No active research experiment".to_string()),
        };

        let target_size = research_def.target_size as usize;
        if card_ids.len() != target_size {
            return Err(format!(
                "Must play exactly {} cards, got {}",
                target_size,
                card_ids.len()
            ));
        }

        // Validate cards are Research cards in hand
        for &cid in &card_ids {
            let card = self
                .library
                .get(cid)
                .ok_or(format!("Card {} not found", cid))?;
            if !matches!(card.kind, CardKind::Research { .. }) {
                return Err(format!("Card {} is not a Research card", cid));
            }
            if card.counts.hand == 0 {
                return Err(format!("Card {} is not in hand", cid));
            }
        }

        // Check for duplicate card IDs that would overdraw
        let mut seen_counts: std::collections::HashMap<usize, u32> =
            std::collections::HashMap::new();
        for &cid in &card_ids {
            *seen_counts.entry(cid).or_insert(0) += 1;
            let card = self
                .library
                .get(cid)
                .ok_or(format!("Card {} not found", cid))?;
            if *seen_counts.get(&cid).unwrap_or(&0) > card.counts.hand {
                return Err(format!(
                    "Card {} played more times than copies in hand",
                    cid
                ));
            }
        }

        // Calculate insight cost for this round: (rounds_played + 1) * base_cost
        let round_num = rounds_played + 1;
        let insight_cost = (round_num as i64) * research_def.base_insight_cost;

        // Determine which discipline's insight to use
        let discipline = self
            .current_research
            .as_ref()
            .map(|p| p.chosen_card.discipline.clone())
            .unwrap_or(Discipline::Research);
        let insight_token = types::TokenType::insight_for_discipline(&discipline);
        let insight_key = types::Token::persistent(insight_token);
        let available = self.token_balances.get(&insight_key).copied().unwrap_or(0);
        if available < insight_cost {
            return Err(format!(
                "Insufficient Insight: need {} but have {}",
                insight_cost, available
            ));
        }

        // Pay costs on premium cards (stamina/health)
        for &cid in &card_ids {
            let card = self
                .library
                .get(cid)
                .ok_or(format!("Card {} not found", cid))?
                .clone();
            if let CardKind::Research { effects } = &card.kind {
                for effect in effects {
                    for cost in &effect.rolled_costs {
                        let cost_val = cost.rolled_percent as i64;
                        let cost_key = types::Token::persistent(cost.token_type.clone());
                        let bal = self.token_balances.get(&cost_key).copied().unwrap_or(0);
                        if bal < cost_val {
                            return Err(format!(
                                "Cannot afford cost: need {} {:?} but have {}",
                                cost_val, cost.token_type, bal
                            ));
                        }
                        *self.token_balances.entry(cost_key).or_insert(0) -= cost_val;
                    }
                }
            }
        }

        // Deduct insight cost
        *self.token_balances.entry(insight_key).or_insert(0) -= insight_cost;

        // Get symbols for each played card
        let card_symbols_list: Vec<Vec<ResearchSymbol>> = card_ids
            .iter()
            .map(|&cid| {
                let card = self.library.get(cid).expect("Card validated above");
                card_symbols(card, &self.library)
            })
            .collect();

        // Run optimal 1:1 matching
        let per_card_yield = optimal_matching(
            &card_symbols_list,
            &hidden_types,
            research_def.position_match_yield,
            research_def.type_match_yield,
        );
        let round_yield: i64 = per_card_yield.iter().sum();

        // Move played cards from hand to discard
        for &cid in &card_ids {
            if cid < self.library.cards.len() {
                let card = &mut self.library.cards[cid];
                if card.counts.hand > 0 {
                    card.counts.hand -= 1;
                    card.counts.discard += 1;
                }
            }
        }

        // Draw replacement cards from deck to hand (1 per card played)
        let mut to_draw = card_ids.len();
        for i in 0..self.library.cards.len() {
            if to_draw == 0 {
                break;
            }
            if matches!(self.library.cards[i].kind, CardKind::Research { .. })
                && self.library.cards[i].counts.deck > 0
            {
                self.library.cards[i].counts.deck -= 1;
                self.library.cards[i].counts.hand += 1;
                to_draw -= 1;
            }
        }

        // Shuffle discard back to deck if deck is empty (for sustainability)
        let deck_count: u32 = self
            .library
            .cards
            .iter()
            .filter(|c| matches!(c.kind, CardKind::Research { .. }))
            .map(|c| c.counts.deck)
            .sum();
        if deck_count == 0 {
            for card in &mut self.library.cards {
                if matches!(card.kind, CardKind::Research { .. }) && card.counts.discard > 0 {
                    card.counts.deck += card.counts.discard;
                    card.counts.discard = 0;
                }
            }
            // Shuffle is implicit via the seeded RNG draw order
        }

        let round_result = ResearchRoundResult {
            cards_played: card_ids,
            per_card_yield,
            round_yield,
            insight_cost,
        };

        if let Some(EncounterState::Research(r)) = &mut self.current_encounter {
            r.accumulated_yield += round_yield;
            r.rounds_played = round_num;
            r.round_history.push(round_result);
        }

        // Ignore unused rng parameter — kept for API consistency
        let _ = rng;

        Ok(())
    }

    /// Conclude the research experiment: apply accumulated yield to research progress.
    pub fn research_conclude_experiment(
        &mut self,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let (accumulated_yield, outcome) = match &self.current_encounter {
            Some(EncounterState::Research(r))
                if r.outcome == EncounterOutcome::Undecided && r.experiment_active =>
            {
                let outcome = if r.accumulated_yield > 0 {
                    EncounterOutcome::PlayerWon
                } else {
                    EncounterOutcome::PlayerLost
                };
                (r.accumulated_yield, outcome)
            }
            _ => return Err("No active research experiment to conclude".to_string()),
        };

        // Apply yield to research progress if there's an active project
        if accumulated_yield > 0 {
            if let Some(project) = &mut self.current_research {
                project.progress += accumulated_yield;

                if project.progress >= project.total_cost {
                    let finished = self
                        .current_research
                        .take()
                        .ok_or("Research project disappeared")?;

                    self.library.add_card(
                        CardKind::Attack {
                            effects: finished.chosen_card.effects.clone(),
                        },
                        CardCounts {
                            library: 0,
                            deck: 0,
                            hand: 0,
                            discard: 0,
                        },
                        rng,
                        vec![finished.chosen_card.discipline.clone()],
                    );
                }
            }
        }

        // Return research cards to deck
        for card in &mut self.library.cards {
            if matches!(card.kind, CardKind::Research { .. }) {
                card.counts.deck += card.counts.hand + card.counts.discard;
                card.counts.hand = 0;
                card.counts.discard = 0;
            }
        }

        if let Some(EncounterState::Research(r)) = &mut self.current_encounter {
            r.experiment_active = false;
            r.outcome = outcome.clone();
        }

        self.record_encounter_finish(types::Discipline::Research, outcome, 1);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
        Ok(())
    }

    pub fn conclude_research_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Research(r)) if r.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active research encounter to conclude".to_string()),
        }
        self.finish_research_encounter();
        Ok(())
    }

    pub fn abort_research_encounter(&mut self) {
        // Return research cards to deck on abort
        for card in &mut self.library.cards {
            if matches!(card.kind, CardKind::Research { .. }) {
                card.counts.deck += card.counts.hand + card.counts.discard;
                card.counts.hand = 0;
                card.counts.discard = 0;
            }
        }
        self.finish_research_encounter();
    }

    fn finish_research_encounter(&mut self) {
        self.record_encounter_finish(types::Discipline::Research, EncounterOutcome::PlayerWon, 1);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }
}
