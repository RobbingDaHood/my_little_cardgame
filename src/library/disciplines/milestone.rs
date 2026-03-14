use super::super::types::{self, *};
use super::super::Library;
use crate::library::game_state::{roll_concrete_effect, GameState};
use std::collections::HashMap;

/// Register initial tier-1 milestone encounter cards for all combat/gathering disciplines.
pub(crate) fn register_milestone_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    register_combat_milestone(lib, rng, 1);
    register_mining_milestone(lib, rng, 1);
    register_herbalism_milestone(lib, rng, 1);
    register_woodcutting_milestone(lib, rng, 1);
    register_fishing_milestone(lib, rng, 1);
}

fn milestone_insight_cost(tier: u32) -> i64 {
    100 * (1i64 << (tier - 1))
}

fn scale_factor(tier: u32) -> f64 {
    1.5f64.powi(tier as i32 - 1)
}

fn scaled(value: i64, tier: u32) -> i64 {
    (value as f64 * scale_factor(tier)).round() as i64
}

fn register_combat_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
) -> usize {
    let scale = scale_factor(tier);
    let enemy_hp = (3000.0 * scale).round() as u64;

    let enemy_damage_id = lib.add_card(
        CardKind::EnemyCardEffect {
            kind: CardEffectKind::LoseTokens {
                token_type: TokenType::Health,
                min: scaled(300, tier),
                max: scaled(500, tier),
                costs: vec![],
                duration: TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![Discipline::Combat],
    );
    let enemy_shield_id = lib.add_card(
        CardKind::EnemyCardEffect {
            kind: CardEffectKind::GainTokens {
                target: EffectTarget::OnSelf,
                token_type: TokenType::Shield,
                cap_min: scaled(200, tier),
                cap_max: scaled(350, tier),
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![Discipline::Combat],
    );
    let enemy_draw_id = lib.add_card(
        CardKind::EnemyCardEffect {
            kind: CardEffectKind::DrawCards {
                attack: 1,
                defence: 1,
                resource: 2,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![Discipline::Combat],
    );

    let combatant_def = CombatantDef {
        initial_tokens: HashMap::from([
            (Token::persistent(TokenType::Health), enemy_hp),
            (Token::persistent(TokenType::MaxHealth), enemy_hp),
        ]),
        attack_deck: vec![EnemyCardDef {
            effects: vec![roll_concrete_effect(rng, enemy_damage_id, lib)],
            counts: DeckCounts {
                deck: 0,
                hand: 12,
                discard: 0,
            },
        }],
        defence_deck: vec![EnemyCardDef {
            effects: vec![roll_concrete_effect(rng, enemy_shield_id, lib)],
            counts: DeckCounts {
                deck: 0,
                hand: 12,
                discard: 0,
            },
        }],
        resource_deck: vec![EnemyCardDef {
            effects: vec![roll_concrete_effect(rng, enemy_draw_id, lib)],
            counts: DeckCounts {
                deck: 0,
                hand: 12,
                discard: 0,
            },
        }],
    };

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Combat { combatant_def }),
        discipline: Discipline::Combat,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Combat],
    )
}

fn register_mining_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
) -> usize {
    let initial_light = (200.0 * scale_factor(tier)).round() as i64;
    let ore_deck = vec![
        OreCard {
            effects: vec![ConcreteEffect {
                effect_id: 0,
                rolled_value: -(scaled(30, tier)),
                rolled_costs: vec![],
                rolled_cap: None,
                rolled_gain_percent: None,
                card_value: None,
            }],
            counts: DeckCounts {
                deck: 0,
                hand: 8,
                discard: 0,
            },
        },
        OreCard {
            effects: vec![ConcreteEffect {
                effect_id: 0,
                rolled_value: -(scaled(60, tier)),
                rolled_costs: vec![],
                rolled_cap: None,
                rolled_gain_percent: None,
                card_value: None,
            }],
            counts: DeckCounts {
                deck: 0,
                hand: 10,
                discard: 0,
            },
        },
        OreCard {
            effects: vec![ConcreteEffect {
                effect_id: 0,
                rolled_value: -(scaled(100, tier)),
                rolled_costs: vec![],
                rolled_cap: None,
                rolled_gain_percent: None,
                card_value: None,
            }],
            counts: DeckCounts {
                deck: 0,
                hand: 4,
                discard: 0,
            },
        },
    ];

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Mining {
            mining_def: MiningDef {
                initial_light_level: initial_light,
                ore_deck,
            },
        }),
        discipline: Discipline::Mining,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Mining],
    )
}

fn register_herbalism_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
) -> usize {
    let plant_hand = vec![
        PlantCard {
            characteristics: vec![PlantCharacteristic::Fragile, PlantCharacteristic::Thorny],
            effects: vec![],
            counts: DeckCounts {
                deck: 0,
                hand: scaled(4, tier) as u32,
                discard: 0,
            },
        },
        PlantCard {
            characteristics: vec![PlantCharacteristic::Aromatic, PlantCharacteristic::Bitter],
            effects: vec![],
            counts: DeckCounts {
                deck: 0,
                hand: scaled(4, tier) as u32,
                discard: 0,
            },
        },
        PlantCard {
            characteristics: vec![PlantCharacteristic::Luminous, PlantCharacteristic::Fragile],
            effects: vec![],
            counts: DeckCounts {
                deck: 0,
                hand: scaled(3, tier) as u32,
                discard: 0,
            },
        },
        PlantCard {
            characteristics: vec![
                PlantCharacteristic::Thorny,
                PlantCharacteristic::Aromatic,
                PlantCharacteristic::Luminous,
            ],
            effects: vec![],
            counts: DeckCounts {
                deck: 0,
                hand: scaled(2, tier) as u32,
                discard: 0,
            },
        },
    ];

    let mut rewards = HashMap::new();
    rewards.insert(Token::persistent(TokenType::Plant), scaled(50, tier));
    rewards.insert(
        Token::persistent(TokenType::HerbalismInsight),
        scaled(10, tier),
    );

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Herbalism {
            herbalism_def: HerbalismDef {
                plant_hand,
                rewards,
            },
        }),
        discipline: Discipline::Herbalism,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Herbalism],
    )
}

fn register_woodcutting_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
) -> usize {
    let max_plays = std::cmp::max(3, 8u32.saturating_sub(tier));
    let mut base_rewards = HashMap::new();
    base_rewards.insert(Token::persistent(TokenType::Lumber), scaled(50, tier));
    base_rewards.insert(
        Token::persistent(TokenType::WoodcuttingInsight),
        scaled(10, tier),
    );

    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Woodcutting {
            woodcutting_def: WoodcuttingDef {
                max_plays,
                base_rewards,
            },
        }),
        discipline: Discipline::Woodcutting,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Woodcutting],
    )
}

fn register_fishing_milestone(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    tier: u32,
) -> usize {
    let fish_deck = vec![
        FishCard {
            value: scaled(40, tier),
            effects: vec![],
            counts: DeckCounts {
                deck: 0,
                hand: 6,
                discard: 0,
            },
        },
        FishCard {
            value: scaled(80, tier),
            effects: vec![],
            counts: DeckCounts {
                deck: 0,
                hand: 4,
                discard: 0,
            },
        },
        FishCard {
            value: scaled(120, tier),
            effects: vec![],
            counts: DeckCounts {
                deck: 0,
                hand: 2,
                discard: 0,
            },
        },
    ];

    let mut rewards = HashMap::new();
    rewards.insert(Token::persistent(TokenType::Fish), scaled(50, tier));
    rewards.insert(
        Token::persistent(TokenType::FishingInsight),
        scaled(10, tier),
    );

    let range_span = std::cmp::max(20, 60i64.saturating_sub(tier as i64 * 10));
    let milestone_def = MilestoneDef {
        inner_encounter_kind: Box::new(EncounterKind::Fishing {
            fishing_def: FishingDef {
                valid_range_min: -range_span,
                valid_range_max: range_span,
                max_turns: scaled(10, tier) as u32,
                win_turns_needed: scaled(6, tier) as u32,
                fish_deck,
                rewards,
            },
        }),
        discipline: Discipline::Fishing,
        tier,
        insight_cost: milestone_insight_cost(tier),
    };

    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Milestone { milestone_def },
        },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 1,
            discard: 0,
        },
        rng,
        vec![Discipline::Fishing],
    )
}

/// Generate 50%-improved versions of every existing PlayerCardEffect for the given discipline.
/// Returns the IDs of newly created effect cards.
pub(crate) fn generate_milestone_reward_effects(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    discipline: &Discipline,
) -> Vec<usize> {
    let existing_effects: Vec<(usize, CardEffectKind)> = lib
        .card_effects_for_discipline(discipline)
        .iter()
        .filter_map(|(id, card)| match &card.kind {
            CardKind::PlayerCardEffect { kind } => Some((*id, kind.clone())),
            _ => None,
        })
        .collect();

    let mut new_ids = Vec::new();
    for (_old_id, kind) in &existing_effects {
        let improved = scale_card_effect_kind(kind, 1.5);
        let new_id = lib.add_card(
            CardKind::PlayerCardEffect { kind: improved },
            CardCounts {
                library: 1,
                deck: 0,
                hand: 0,
                discard: 0,
            },
            rng,
            vec![discipline.clone()],
        );
        new_ids.push(new_id);
    }
    new_ids
}

fn scale_card_effect_kind(kind: &CardEffectKind, factor: f64) -> CardEffectKind {
    match kind {
        CardEffectKind::GainTokens {
            target,
            token_type,
            cap_min,
            cap_max,
            gain_min_percent,
            gain_max_percent,
            costs,
            duration,
        } => CardEffectKind::GainTokens {
            target: target.clone(),
            token_type: token_type.clone(),
            cap_min: ((*cap_min as f64) * factor).round() as i64,
            cap_max: ((*cap_max as f64) * factor).round() as i64,
            gain_min_percent: *gain_min_percent,
            gain_max_percent: *gain_max_percent,
            costs: costs.clone(),
            duration: duration.clone(),
        },
        CardEffectKind::LoseTokens {
            token_type,
            min,
            max,
            costs,
            duration,
        } => CardEffectKind::LoseTokens {
            token_type: token_type.clone(),
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
            costs: costs.clone(),
            duration: duration.clone(),
        },
        CardEffectKind::DrawCards {
            attack,
            defence,
            resource,
        } => CardEffectKind::DrawCards {
            attack: attack + 1,
            defence: *defence,
            resource: *resource,
        },
        CardEffectKind::Insight { min, max } => CardEffectKind::Insight {
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
        },
        CardEffectKind::WoodcuttingChop {
            chop_type,
            min_value,
            max_value,
            costs,
        } => CardEffectKind::WoodcuttingChop {
            chop_type: chop_type.clone(),
            min_value: ((*min_value as f64) * factor).round() as u32,
            max_value: ((*max_value as f64) * factor).round() as u32,
            costs: costs.clone(),
        },
        CardEffectKind::HerbalismMatch { match_mode, costs } => CardEffectKind::HerbalismMatch {
            match_mode: match_mode.clone(),
            costs: costs.clone(),
        },
        CardEffectKind::FishingValue { min, max, costs } => CardEffectKind::FishingValue {
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
            costs: costs.clone(),
        },
        CardEffectKind::CraftingReduction {
            token_type,
            min,
            max,
            costs,
        } => CardEffectKind::CraftingReduction {
            token_type: token_type.clone(),
            min: ((*min as f64) * factor).round() as i64,
            max: ((*max as f64) * factor).round() as i64,
            costs: costs.clone(),
        },
        CardEffectKind::ResearchProbe { symbols, costs } => CardEffectKind::ResearchProbe {
            symbols: symbols.clone(),
            costs: costs.clone(),
        },
    }
}

/// Generate 3 variations of a next-tier milestone encounter for the given discipline.
/// Returns the card IDs.
pub(crate) fn generate_next_tier_milestone_encounters(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    discipline: &Discipline,
    current_tier: u32,
) -> Vec<usize> {
    let next_tier = current_tier + 1;
    let mut ids = Vec::with_capacity(3);
    for _ in 0..3 {
        let id = match discipline {
            Discipline::Combat => register_combat_milestone(lib, rng, next_tier),
            Discipline::Mining => register_mining_milestone(lib, rng, next_tier),
            Discipline::Herbalism => register_herbalism_milestone(lib, rng, next_tier),
            Discipline::Woodcutting => register_woodcutting_milestone(lib, rng, next_tier),
            Discipline::Fishing => register_fishing_milestone(lib, rng, next_tier),
            _ => continue,
        };
        ids.push(id);
    }
    ids
}

// ---- GameState methods for milestone encounters ----

impl GameState {
    /// Start a milestone encounter: deduct MilestoneInsight, delegate to inner discipline.
    pub fn start_milestone_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();

        let milestone_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Milestone { milestone_def },
            } => milestone_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a milestone encounter",
                    encounter_card_id
                ))
            }
        };

        // Deduct MilestoneInsight cost
        let insight_key = Token::persistent(TokenType::MilestoneInsight);
        let balance = self.token_balances.get(&insight_key).copied().unwrap_or(0);
        if balance < milestone_def.insight_cost {
            return Err(format!(
                "Insufficient MilestoneInsight: have {}, need {}",
                balance, milestone_def.insight_cost
            ));
        }
        *self.token_balances.entry(insight_key).or_insert(0) -= milestone_def.insight_cost;

        // Start the inner encounter
        let discipline = milestone_def.discipline.clone();
        let tier = milestone_def.tier;
        match milestone_def.inner_encounter_kind.as_ref() {
            EncounterKind::Combat { combatant_def } => {
                self.start_combat_inner(encounter_card_id, combatant_def.clone(), rng)?;
            }
            EncounterKind::Mining { mining_def } => {
                self.start_mining_inner(encounter_card_id, mining_def.clone(), rng)?;
            }
            EncounterKind::Herbalism { herbalism_def } => {
                self.start_herbalism_inner(encounter_card_id, herbalism_def.clone(), rng)?;
            }
            EncounterKind::Woodcutting { woodcutting_def } => {
                self.start_woodcutting_inner(encounter_card_id, woodcutting_def.clone(), rng)?;
            }
            EncounterKind::Fishing { fishing_def } => {
                self.start_fishing_inner(encounter_card_id, fishing_def.clone(), rng)?;
            }
            _ => return Err("Unsupported milestone inner encounter type".to_string()),
        }

        // Wrap the inner state in MilestoneEncounterState
        let inner_state = self
            .current_encounter
            .take()
            .ok_or("Inner encounter failed to initialize")?;

        self.current_encounter = Some(EncounterState::Milestone(MilestoneEncounterState {
            encounter_card_id,
            inner_state: Box::new(inner_state),
            discipline,
            tier,
            outcome: EncounterOutcome::Undecided,
        }));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Delegate card play to the inner encounter within a milestone.
    pub fn resolve_milestone_play_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        // Extract the inner state, resolve the play, then re-wrap
        let milestone = match self.current_encounter.take() {
            Some(EncounterState::Milestone(m)) => m,
            other => {
                self.current_encounter = other;
                return Err("Not in a milestone encounter".to_string());
            }
        };

        // Temporarily set the inner state as current
        self.current_encounter = Some(*milestone.inner_state);

        // Delegate to the appropriate discipline's play resolution
        let result = match &self.current_encounter {
            Some(EncounterState::Combat(_)) => {
                self.resolve_player_card(card_id, rng)?;
                if self.current_encounter.is_some() {
                    self.resolve_enemy_play(rng)?;
                    if self.current_encounter.is_some() {
                        self.advance_combat_phase()?;
                    }
                }
                Ok(())
            }
            Some(EncounterState::Mining(_)) => self.resolve_player_mining_card(card_id, rng),
            Some(EncounterState::Herbalism(_)) => self.resolve_player_herbalism_card(card_id, rng),
            Some(EncounterState::Woodcutting(_)) => {
                self.resolve_player_woodcutting_card(card_id, rng)
            }
            Some(EncounterState::Fishing(_)) => self.resolve_player_fishing_card(card_id, rng),
            _ => Err("Unsupported inner encounter type for milestone".to_string()),
        };

        // Check if inner encounter ended (combat sets current_encounter = None on finish)
        let inner_finished = self.current_encounter.is_none()
            || self
                .current_encounter
                .as_ref()
                .map(|e| e.is_finished())
                .unwrap_or(false);

        if inner_finished {
            // The inner discipline's finish logic already ran (including
            // setting encounter_phase to Scouting and recording metrics).
            // We need to override the behaviour for milestones.
            let inner_outcome = if self.current_encounter.is_none() {
                // Combat clears current_encounter on finish; check last_encounter_result
                self.encounter_results
                    .last()
                    .cloned()
                    .unwrap_or(EncounterOutcome::PlayerLost)
            } else {
                self.current_encounter
                    .as_ref()
                    .map(|e| e.outcome().clone())
                    .unwrap_or(EncounterOutcome::PlayerLost)
            };

            self.handle_milestone_finish(
                milestone.encounter_card_id,
                milestone.discipline,
                milestone.tier,
                inner_outcome,
                rng,
            );
        } else {
            // Re-wrap the inner state
            let inner_state = self.current_encounter.take().ok_or("Inner state lost")?;
            self.current_encounter = Some(EncounterState::Milestone(MilestoneEncounterState {
                encounter_card_id: milestone.encounter_card_id,
                inner_state: Box::new(inner_state),
                discipline: milestone.discipline,
                tier: milestone.tier,
                outcome: EncounterOutcome::Undecided,
            }));
            self.encounter_phase = types::EncounterPhase::InEncounter;
        }

        result
    }

    fn handle_milestone_finish(
        &mut self,
        encounter_card_id: usize,
        discipline: Discipline,
        tier: u32,
        outcome: EncounterOutcome,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) {
        self.current_encounter = None;

        if outcome == EncounterOutcome::PlayerWon {
            // Generate reward effects
            generate_milestone_reward_effects(&mut self.library, rng, &discipline);

            // Generate 3 next-tier scouting choices
            let choices =
                generate_next_tier_milestone_encounters(&mut self.library, rng, &discipline, tier);
            self.milestone_scouting_choices = choices;

            // Remove the old milestone card from library (it's been beaten)
            self.library.delete_card(encounter_card_id);

            self.encounter_phase = types::EncounterPhase::MilestoneScouting;
        } else {
            // Loss: return encounter card to hand, go to NoEncounter
            let _ = self.library.return_to_hand(encounter_card_id);
            self.encounter_phase = types::EncounterPhase::NoEncounter;
        }

        self.check_player_death();
    }

    /// After winning a milestone, player picks one of 3 next-tier encounters.
    pub fn milestone_pick_scouting_choice(
        &mut self,
        card_id: usize,
        _rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        if self.encounter_phase != types::EncounterPhase::MilestoneScouting {
            return Err("Not in MilestoneScouting phase".to_string());
        }

        if !self.milestone_scouting_choices.contains(&card_id) {
            return Err(format!(
                "Card {} is not one of the milestone scouting choices",
                card_id
            ));
        }

        // Delete the unchosen cards
        for &cid in &self.milestone_scouting_choices {
            if cid != card_id {
                self.library.delete_card(cid);
            }
        }

        // The chosen card is already in the library with hand count 1
        self.milestone_scouting_choices.clear();
        self.encounter_phase = types::EncounterPhase::NoEncounter;
        Ok(())
    }

    /// Abort a milestone encounter (treated as loss).
    pub fn abort_milestone_encounter(&mut self) {
        if let Some(EncounterState::Milestone(m)) = &self.current_encounter {
            let enc_id = m.encounter_card_id;
            let discipline = m.discipline.clone();
            let rounds = m.inner_state.round();
            self.record_encounter_finish(discipline, EncounterOutcome::PlayerLost, rounds);
            self.current_encounter = None;
            let _ = self.library.return_to_hand(enc_id);
            self.encounter_phase = types::EncounterPhase::NoEncounter;
            self.check_player_death();
        }
    }

    // ---- Inner encounter start helpers (bypass card lookup, use def directly) ----

    fn start_combat_inner(
        &mut self,
        encounter_card_id: usize,
        combatant_def: CombatantDef,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        use crate::library::game_state::deck_shuffle_hand;

        let mut attack_deck = combatant_def.attack_deck;
        let mut defence_deck = combatant_def.defence_deck;
        let mut resource_deck = combatant_def.resource_deck;
        deck_shuffle_hand(rng, &mut attack_deck);
        deck_shuffle_hand(rng, &mut defence_deck);
        deck_shuffle_hand(rng, &mut resource_deck);

        let state = CombatEncounterState {
            round: 1,
            phase: CombatPhase::Defending,
            enemy_tokens: combatant_def
                .initial_tokens
                .iter()
                .map(|(k, v)| (k.clone(), *v as i64))
                .collect(),
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            enemy_attack_deck: attack_deck,
            enemy_defence_deck: defence_deck,
            enemy_resource_deck: resource_deck,
        };
        self.current_encounter = Some(EncounterState::Combat(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    fn start_mining_inner(
        &mut self,
        encounter_card_id: usize,
        mining_def: MiningDef,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        use crate::library::game_state::deck_shuffle_hand;

        let mut ore_deck = mining_def.ore_deck;
        deck_shuffle_hand(rng, &mut ore_deck);

        let mut encounter_tokens = HashMap::new();
        encounter_tokens.insert(
            Token::persistent(TokenType::MiningLightLevel),
            mining_def.initial_light_level,
        );
        encounter_tokens.insert(Token::persistent(TokenType::MiningYield), 0);

        let state = MiningEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            ore_deck,
            encounter_tokens,
        };
        self.current_encounter = Some(EncounterState::Mining(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    fn start_herbalism_inner(
        &mut self,
        encounter_card_id: usize,
        herbalism_def: HerbalismDef,
        _rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let state = HerbalismEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            plant_hand: herbalism_def.plant_hand,
            rewards: herbalism_def.rewards,
        };
        self.current_encounter = Some(EncounterState::Herbalism(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    fn start_woodcutting_inner(
        &mut self,
        encounter_card_id: usize,
        woodcutting_def: WoodcuttingDef,
        _rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let state = WoodcuttingEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            played_cards: Vec::new(),
            max_plays: woodcutting_def.max_plays,
            pattern_name: None,
            pattern_multiplier: None,
            base_rewards: woodcutting_def.base_rewards,
        };
        self.current_encounter = Some(EncounterState::Woodcutting(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    fn start_fishing_inner(
        &mut self,
        encounter_card_id: usize,
        fishing_def: FishingDef,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        use crate::library::game_state::deck_shuffle_hand;

        let mut fish_deck = fishing_def.fish_deck;
        deck_shuffle_hand(rng, &mut fish_deck);

        let mut encounter_tokens = HashMap::new();
        encounter_tokens.insert(
            Token::persistent(TokenType::FishingRangeMin),
            fishing_def.valid_range_min,
        );
        encounter_tokens.insert(
            Token::persistent(TokenType::FishingRangeMax),
            fishing_def.valid_range_max,
        );

        let state = FishingEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            turns_won: 0,
            max_turns: fishing_def.max_turns,
            win_turns_needed: fishing_def.win_turns_needed,
            valid_range_min: fishing_def.valid_range_min,
            valid_range_max: fishing_def.valid_range_max,
            fish_deck,
            rewards: fishing_def.rewards,
            encounter_tokens,
        };
        self.current_encounter = Some(EncounterState::Fishing(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }
}
