use super::action_log::ActionLog;
use super::types::{
    ActionEntry, ActionPayload, CardCounts, CardKind, ConcreteEffect, ConcreteEffectCost,
    EncounterKind, EncounterOutcome, EncounterState, HasDeckCounts,
};
use super::Library;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

pub(crate) fn roll_range(rng: &mut rand_pcg::Lcg64Xsh32, min: i64, max: i64) -> i64 {
    use rand::RngCore;
    if min == max {
        return min;
    }
    let (lo, hi) = if min < max { (min, max) } else { (max, min) };
    let range = (hi - lo + 1) as u64;
    lo + (rng.next_u64() % range) as i64
}

pub(crate) fn roll_range_u32(rng: &mut rand_pcg::Lcg64Xsh32, min: u32, max: u32) -> u32 {
    use rand::RngCore;
    if min == max {
        return min;
    }
    let (lo, hi) = if min < max { (min, max) } else { (max, min) };
    let range = (hi - lo + 1) as u64;
    lo + (rng.next_u64() % range) as u32
}

pub(crate) fn roll_concrete_effect(
    rng: &mut rand_pcg::Lcg64Xsh32,
    effect_id: usize,
    library: &Library,
) -> ConcreteEffect {
    let kind = library.resolve_effect(effect_id);
    let (rolled_value, rolled_costs, rolled_cap, rolled_gain_percent) = match kind {
        Some(super::types::CardEffectKind::GainTokens {
            cap_min,
            cap_max,
            gain_min_percent,
            gain_max_percent,
            costs,
            ..
        }) => {
            let r_cap = roll_range(rng, cap_min, cap_max);
            let r_gain = roll_range_u32(rng, gain_min_percent, gain_max_percent);
            let value = r_cap * r_gain as i64 / 100;
            let costs = costs
                .iter()
                .map(|c| ConcreteEffectCost {
                    token_type: c.token_type.clone(),
                    rolled_percent: roll_range_u32(rng, c.min_percent, c.max_percent),
                })
                .collect();
            (value, costs, Some(r_cap), Some(r_gain))
        }
        Some(super::types::CardEffectKind::LoseTokens {
            min, max, costs, ..
        }) => {
            let value = roll_range(rng, min, max);
            let costs = costs
                .iter()
                .map(|c| ConcreteEffectCost {
                    token_type: c.token_type.clone(),
                    rolled_percent: roll_range_u32(rng, c.min_percent, c.max_percent),
                })
                .collect();
            (value, costs, None, None)
        }
        _ => (0, vec![], None, None),
    };
    ConcreteEffect {
        effect_id,
        rolled_value,
        rolled_costs,
        rolled_cap,
        rolled_gain_percent,
    }
}

fn initialize_library(rng: &mut rand_pcg::Lcg64Xsh32) -> Library {
    let mut lib = Library::new();

    // ---- Shared PlayerCardEffect deck entries (templates with ranges) ----

    // Player "deal damage" effect (range: 400-600)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::LoseTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: 400,
                max: 600,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Player "grant shield" effect (range: 200-400)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                cap_min: 200,
                cap_max: 400,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Player "grant stamina" effect (range: 150-250)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                cap_min: 150,
                cap_max: 250,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Player "draw 1 attack, 1 defence, 2 resource" effect
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::DrawCards {
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
    );

    // ---- Shared EnemyCardEffect deck entries (templates with ranges) ----

    // Enemy "deal damage" effect (range: 200-400)
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::LoseTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                min: 200,
                max: 400,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Enemy "grant shield" effect (range: 150-250)
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                cap_min: 150,
                cap_max: 250,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Enemy "grant stamina" effect (range: 80-120)
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::GainTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                cap_min: 80,
                cap_max: 120,
                gain_min_percent: 100,
                gain_max_percent: 100,
                costs: vec![],
                duration: super::types::TokenLifecycle::PersistentCounter,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // Enemy "draw 1 attack, 1 defence, 2 resource" effect
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::DrawCards {
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
    );

    // ---- Discipline-specific cards ----
    super::disciplines::combat::register_combat_cards(&mut lib, rng);
    super::disciplines::mining::register_mining_cards(&mut lib, rng);
    super::disciplines::herbalism::register_herbalism_cards(&mut lib, rng);
    super::disciplines::woodcutting::register_woodcutting_cards(&mut lib, rng);
    super::disciplines::fishing::register_fishing_cards(&mut lib, rng);
    super::disciplines::rest::register_rest_cards(&mut lib, rng);
    super::disciplines::crafting::register_crafting_cards(&mut lib, rng);

    if let Err(errors) = lib.validate_card_effects() {
        panic!("Library card effect validation failed: {:?}", errors);
    }

    lib
}

/// Minimal in-memory game state driven by the library's mutator API.
#[derive(Debug, Clone)]
pub struct GameState {
    pub action_log: std::sync::Arc<ActionLog>,
    pub token_balances: HashMap<super::types::Token, i64>,
    pub library: Library,
    pub current_encounter: Option<EncounterState>,
    pub encounter_phase: super::types::EncounterPhase,
    pub last_encounter_result: Option<EncounterOutcome>,
    pub encounter_results: Vec<EncounterOutcome>,
}

impl GameState {
    pub fn new() -> Self {
        use rand::SeedableRng;
        let mut rng = rand_pcg::Lcg64Xsh32::from_entropy();
        Self::new_with_rng(&mut rng)
    }

    pub fn new_with_rng(rng: &mut rand_pcg::Lcg64Xsh32) -> Self {
        let mut balances = HashMap::new();
        for id in super::types::TokenType::all() {
            balances.insert(super::types::Token::persistent(id), 0i64);
        }
        // Default Foresight controls area deck hand size
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::Foresight),
            3,
        );
        // Durabilities scaled by ~100x (100→10000)
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::MiningDurability),
            10000,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::HerbalismDurability),
            10000,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::WoodcuttingDurability),
            10000,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::FishingDurability),
            10000,
        );
        // Starting stamina
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::Stamina),
            1000,
        );
        // Starting health
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::Health),
            1000,
        );
        // Max handsize tokens (player decks)
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::AttackMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::DefenceMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::ResourceMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::MiningMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::HerbalismMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::WoodcuttingMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::FishingMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::RestMaxHand),
            5,
        );
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::CraftingMaxHand),
            5,
        );
        let _action_log = match std::env::var("ACTION_LOG_FILE") {
            Ok(path) => {
                #[allow(clippy::manual_unwrap_or_default)]
                let mut log = match super::action_log::ActionLog::load_from_file(&path) {
                    Ok(l) => l,
                    Err(_) => ActionLog::new(),
                };
                if let Ok(writer) =
                    crate::action::persistence::FileWriter::new(std::path::PathBuf::from(&path))
                {
                    log.set_writer(Some(writer));
                }
                log
            }
            Err(_) => ActionLog::new(),
        };
        Self {
            action_log: std::sync::Arc::new(ActionLog::new()),
            token_balances: balances,
            library: initialize_library(rng),
            current_encounter: None,
            encounter_phase: super::types::EncounterPhase::NoEncounter,
            last_encounter_result: None,
            encounter_results: Vec::new(),
        }
    }

    /// Append an action to the action log with optional metadata; returns the appended entry.
    pub fn append_action(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
        self.action_log.append(action_type, payload)
    }

    /// Check if player can pay all costs on a card's effects. Deducts costs if affordable.
    pub(crate) fn check_and_deduct_costs(
        effects: &[ConcreteEffect],
        token_balances: &mut HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        Self::preview_costs(effects, token_balances)?;
        // Deduct costs (we know they're affordable from preview)
        for effect in effects {
            for cost in &effect.rolled_costs {
                let cost_amount =
                    (effect.rolled_value.unsigned_abs() * cost.rolled_percent as u64 / 100) as i64;
                let entry = super::types::token_entry_by_type(token_balances, &cost.token_type);
                *entry -= cost_amount;
            }
        }
        Ok(())
    }

    /// Check if player can afford all costs without deducting. Used for pre-validation.
    pub fn preview_costs(
        effects: &[ConcreteEffect],
        token_balances: &HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        let mut total_costs: HashMap<super::types::TokenType, i64> = HashMap::new();
        for effect in effects {
            for cost in &effect.rolled_costs {
                let cost_amount =
                    (effect.rolled_value.unsigned_abs() * cost.rolled_percent as u64 / 100) as i64;
                *total_costs.entry(cost.token_type.clone()).or_insert(0) += cost_amount;
            }
        }
        for (token_type, cost_amount) in &total_costs {
            let balance = super::types::token_balance_by_type(token_balances, token_type);
            if balance < *cost_amount {
                return Err(format!(
                    "Insufficient {:?}: need {} but have {}",
                    token_type, cost_amount, balance
                ));
            }
        }
        Ok(())
    }

    /// Check and deduct a list of gathering costs. All costs must be affordable.
    pub(crate) fn check_and_deduct_gathering_costs(
        costs: &[super::types::TokenAmount],
        token_balances: &mut HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        Self::preview_gathering_costs(costs, token_balances)?;
        for cost in costs {
            if cost.amount > 0 {
                let entry = super::types::token_entry_by_type(token_balances, &cost.token_type);
                *entry -= cost.amount;
            }
        }
        Ok(())
    }

    /// Check if player can afford gathering costs without deducting.
    pub fn preview_gathering_costs(
        costs: &[super::types::TokenAmount],
        token_balances: &HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        for cost in costs {
            if cost.amount <= 0 {
                continue;
            }
            let balance = super::types::token_balance_by_type(token_balances, &cost.token_type);
            if balance < cost.amount {
                return Err(format!(
                    "Insufficient {:?}: need {} but have {}",
                    cost.token_type, cost.amount, balance
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn all_gathering_hand_cards_unpayable(
        &self,
        cost_extractor: impl Fn(&super::types::CardKind) -> Option<&Vec<super::types::TokenAmount>>,
    ) -> bool {
        let hand_cards: Vec<_> = self
            .library
            .cards
            .iter()
            .filter(|c| c.counts.hand > 0 && cost_extractor(&c.kind).is_some())
            .collect();
        if hand_cards.is_empty() {
            return false;
        }
        hand_cards.iter().all(|card| {
            let costs = cost_extractor(&card.kind).unwrap();
            let (pre_play_costs, _) = super::types::split_token_amounts(costs);
            if pre_play_costs.is_empty() {
                return false;
            }
            Self::preview_gathering_costs(&pre_play_costs, &self.token_balances).is_err()
        })
    }

    /// Draw player cards from deck to hand per card type, recycling discard if needed.
    pub(crate) fn draw_player_cards_by_type(
        &mut self,
        attack: u32,
        defence: u32,
        resource: u32,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) {
        self.draw_player_cards_of_kind(
            attack,
            |k| matches!(k, CardKind::Attack { .. }),
            rng,
            Some(super::types::TokenType::AttackMaxHand),
        );
        self.draw_player_cards_of_kind(
            defence,
            |k| matches!(k, CardKind::Defence { .. }),
            rng,
            Some(super::types::TokenType::DefenceMaxHand),
        );
        self.draw_player_cards_of_kind(
            resource,
            |k| matches!(k, CardKind::Resource { .. }),
            rng,
            Some(super::types::TokenType::ResourceMaxHand),
        );
    }

    /// Draw `count` player cards of a specific kind from deck to hand.
    /// Recycles discard→deck for cards matching `kind_filter` when deck is empty.
    /// Respects max handsize token if provided.
    pub(crate) fn draw_player_cards_of_kind(
        &mut self,
        count: u32,
        kind_filter: fn(&CardKind) -> bool,
        rng: &mut rand_pcg::Lcg64Xsh32,
        max_hand_token: Option<super::types::TokenType>,
    ) {
        use rand::RngCore;
        for _ in 0..count {
            let drawable: Vec<usize> = self
                .library
                .cards
                .iter()
                .enumerate()
                .filter(|(_, c)| c.counts.deck > 0 && kind_filter(&c.kind))
                .map(|(i, _)| i)
                .collect();
            if drawable.is_empty() {
                // Recycle discard→deck for this card type
                for card in self.library.cards.iter_mut() {
                    if kind_filter(&card.kind) && card.counts.discard > 0 {
                        card.counts.deck += card.counts.discard;
                        card.counts.discard = 0;
                    }
                }
                let drawable: Vec<usize> = self
                    .library
                    .cards
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.counts.deck > 0 && kind_filter(&c.kind))
                    .map(|(i, _)| i)
                    .collect();
                if drawable.is_empty() {
                    return;
                }
                let pick = (rng.next_u64() as usize) % drawable.len();
                if self.handsize_reached(&kind_filter, &max_hand_token) {
                    continue;
                }
                let _ = self.library.draw(drawable[pick]);
            } else {
                let pick = (rng.next_u64() as usize) % drawable.len();
                if self.handsize_reached(&kind_filter, &max_hand_token) {
                    continue;
                }
                let _ = self.library.draw(drawable[pick]);
            }
        }
    }

    fn handsize_reached(
        &self,
        kind_filter: &fn(&CardKind) -> bool,
        max_hand_token: &Option<super::types::TokenType>,
    ) -> bool {
        if let Some(ref token) = max_hand_token {
            let max_hand = super::types::token_balance_by_type(&self.token_balances, token);
            let current_hand: u32 = self
                .library
                .cards
                .iter()
                .filter(|c| kind_filter(&c.kind))
                .map(|c| c.counts.hand)
                .sum();
            current_hand as i64 >= max_hand
        } else {
            false
        }
    }

    /// Abort a non-combat encounter: mark as lost, transition to Scouting.
    pub fn abort_encounter(&mut self) {
        self.last_encounter_result = Some(EncounterOutcome::PlayerLost);
        self.encounter_results.push(EncounterOutcome::PlayerLost);
        self.current_encounter = None;
        self.encounter_phase = super::types::EncounterPhase::Scouting;
    }

    /// Check if the player has died (Health <= 0) and apply death consequences:
    /// lose all gathering material tokens, reset health and stamina, increment deaths counter.
    pub(crate) fn check_player_death(&mut self) {
        let health_key = super::types::Token::persistent(super::types::TokenType::Health);
        let health = self.token_balances.get(&health_key).copied().unwrap_or(0);
        if health > 0 {
            return;
        }

        // Reset gathering material tokens to 0
        for (token, balance) in &mut self.token_balances {
            if token.token_type.is_gathering_material() {
                *balance = 0;
            }
        }

        // Reset health and stamina to initial values
        self.token_balances.insert(health_key, 1000);
        self.token_balances.insert(
            super::types::Token::persistent(super::types::TokenType::Stamina),
            1000,
        );

        // Increment player deaths counter
        let deaths_key = super::types::Token::persistent(super::types::TokenType::PlayerDeaths);
        let deaths = self.token_balances.entry(deaths_key).or_insert(0);
        *deaths += 1;
    }

    /// Reconstruct state from an existing action log.
    /// The RNG is initialized from the first `SetSeed` entry in the log.
    pub fn replay_from_log(log: &ActionLog) -> Self {
        use rand::SeedableRng;

        let mut gs = GameState::new();
        let mut rng = rand_pcg::Lcg64Xsh32::from_seed([0u8; 16]);

        for e in log.entries() {
            match &e.payload {
                ActionPayload::SetSeed { seed } => {
                    let mut seed_bytes = [0u8; 16];
                    seed_bytes[0..8].copy_from_slice(&seed.to_le_bytes());
                    seed_bytes[8..16].copy_from_slice(&seed.to_le_bytes());
                    rng = rand_pcg::Lcg64Xsh32::from_seed(seed_bytes);
                    let new_gs = GameState::new();
                    gs.library = new_gs.library;
                    gs.token_balances = new_gs.token_balances;
                    gs.current_encounter = None;
                    gs.encounter_phase = new_gs.encounter_phase;
                    gs.last_encounter_result = None;
                    gs.encounter_results.clear();
                }
                ActionPayload::DrawEncounter { encounter_id } => {
                    if let Ok(card_id) = encounter_id.parse::<usize>() {
                        let health_key =
                            super::types::Token::persistent(super::types::TokenType::Health);
                        if gs.token_balances.get(&health_key).copied().unwrap_or(0) == 0 {
                            gs.token_balances.insert(health_key, 20);
                        }
                        let _ = gs.library.play(card_id);
                        // Dispatch based on encounter kind
                        if let Some(lib_card) = gs.library.get(card_id) {
                            match &lib_card.kind {
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Mining { .. },
                                } => {
                                    let _ = gs.start_mining_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Herbalism { .. },
                                } => {
                                    let _ = gs.start_herbalism_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Woodcutting { .. },
                                } => {
                                    let _ = gs.start_woodcutting_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Fishing { .. },
                                } => {
                                    let _ = gs.start_fishing_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Rest { .. },
                                } => {
                                    let _ = gs.start_rest_encounter(card_id, &mut rng);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Crafting { .. },
                                } => {
                                    let _ = gs.start_crafting_encounter(card_id, &mut rng);
                                }
                                _ => {
                                    let _ = gs.start_combat(card_id, &mut rng);
                                }
                            }
                        }
                    }
                }
                ActionPayload::PlayCard { card_id } => {
                    // Rest encounters handle library.play() internally
                    let is_rest = matches!(&gs.current_encounter, Some(EncounterState::Rest(_)));
                    if !is_rest {
                        let _ = gs.library.play(*card_id);
                    }
                    match &gs.current_encounter {
                        Some(EncounterState::Combat(_)) => {
                            let _ = gs.resolve_player_card(*card_id, &mut rng);
                            if gs.current_encounter.is_some() {
                                let _ = gs.resolve_enemy_play(&mut rng);
                                if gs.current_encounter.is_some() {
                                    let _ = gs.advance_combat_phase();
                                }
                            }
                        }
                        Some(EncounterState::Mining(_)) => {
                            let _ = gs.resolve_player_mining_card(*card_id, &mut rng);
                        }
                        Some(EncounterState::Herbalism(_)) => {
                            let _ = gs.resolve_player_herbalism_card(*card_id, &mut rng);
                        }
                        Some(EncounterState::Woodcutting(_)) => {
                            let _ = gs.resolve_player_woodcutting_card(*card_id, &mut rng);
                        }
                        Some(EncounterState::Fishing(_)) => {
                            let _ = gs.resolve_player_fishing_card(*card_id, &mut rng);
                        }
                        Some(EncounterState::Rest(_)) => {
                            let _ = gs.resolve_rest_card_play(*card_id, &mut rng);
                        }
                        Some(EncounterState::Crafting(_)) => {
                            let _ = gs.resolve_crafting_play_card(*card_id, &mut rng);
                        }
                        None => {}
                    }
                }
                ActionPayload::ApplyScouting { .. } => {
                    if let Some(ref enc) = gs.current_encounter {
                        let enc_id = enc.encounter_card_id();
                        let _ = gs.library.return_to_deck(enc_id);
                    }
                    let foresight = gs
                        .token_balances
                        .get(&super::types::Token::persistent(
                            super::types::TokenType::Foresight,
                        ))
                        .copied()
                        .unwrap_or(3) as usize;
                    gs.library.encounter_draw_to_hand(foresight);
                    gs.encounter_phase = super::types::EncounterPhase::NoEncounter;
                }
                ActionPayload::AbortEncounter => {
                    if matches!(&gs.current_encounter, Some(EncounterState::Rest(_))) {
                        gs.abort_rest_encounter();
                    } else if matches!(&gs.current_encounter, Some(EncounterState::Crafting(_))) {
                        let _ = gs.abort_crafting_encounter();
                    } else {
                        gs.abort_encounter();
                    }
                }
                ActionPayload::ConcludeEncounter => match &gs.current_encounter {
                    Some(EncounterState::Mining(_)) => {
                        let _ = gs.conclude_mining_encounter();
                    }
                    Some(EncounterState::Crafting(_)) => {
                        let _ = gs.conclude_crafting_encounter();
                    }
                    _ => {}
                },
                ActionPayload::CraftSwap { from_id, to_id } => {
                    let _ = gs.resolve_crafting_swap(*from_id, *to_id);
                }
                ActionPayload::CraftCard { target_card_id } => {
                    let _ = gs.resolve_crafting_start_craft(*target_card_id);
                }
                ActionPayload::CraftDurability { discipline } => {
                    let _ = gs.resolve_crafting_add_durability(discipline);
                }
            }
            match gs.action_log.entries.lock() {
                Ok(mut g) => g.push(e.clone()),
                Err(err) => err.into_inner().push(e.clone()),
            };
            let cur = gs.action_log.seq.load(Ordering::SeqCst);
            if cur < e.seq {
                gs.action_log.seq.store(e.seq, Ordering::SeqCst);
            }
        }
        gs
    }

    /// Graceful shutdown helper to flush and close any background writer.
    pub fn shutdown(&self) {
        if let Some(w) = &self.action_log.writer {
            w.close();
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn deck_draw_random<T: HasDeckCounts>(rng: &mut rand_pcg::Lcg64Xsh32, cards: &mut [T]) {
    use rand::RngCore;
    let total_deck: u32 = cards.iter().map(|c| c.deck_count()).sum();
    if total_deck == 0 {
        let total_discard: u32 = cards.iter().map(|c| c.discard_count()).sum();
        if total_discard == 0 {
            return;
        }
        for card in cards.iter_mut() {
            *card.deck_count_mut() += card.discard_count();
            *card.discard_count_mut() = 0;
        }
    }
    let total_deck: u32 = cards.iter().map(|c| c.deck_count()).sum();
    if total_deck == 0 {
        return;
    }
    let mut pick = (rng.next_u64() as u32) % total_deck;
    for card in cards.iter_mut() {
        if pick < card.deck_count() {
            *card.deck_count_mut() -= 1;
            *card.hand_count_mut() += 1;
            return;
        }
        pick -= card.deck_count();
    }
}

pub(crate) fn deck_shuffle_hand<T: HasDeckCounts>(rng: &mut rand_pcg::Lcg64Xsh32, cards: &mut [T]) {
    let target_hand: u32 = cards.iter().map(|c| c.hand_count()).sum();
    for card in cards.iter_mut() {
        *card.deck_count_mut() += card.hand_count();
        *card.hand_count_mut() = 0;
    }
    for _ in 0..target_hand {
        deck_draw_random(rng, cards);
    }
}

/// Pick a random card from hand, move it to discard. Returns the index of the picked card, or None.
pub(crate) fn deck_play_random<T: HasDeckCounts>(
    rng: &mut rand_pcg::Lcg64Xsh32,
    cards: &mut [T],
) -> Option<usize> {
    use rand::RngCore;
    let total_hand: u32 = cards.iter().map(|c| c.hand_count()).sum();
    if total_hand == 0 {
        let total_discard: u32 = cards.iter().map(|c| c.discard_count()).sum();
        if total_discard == 0 {
            return None;
        }
        for card in cards.iter_mut() {
            *card.hand_count_mut() += card.discard_count();
            *card.discard_count_mut() = 0;
        }
    }
    let total_hand: u32 = cards.iter().map(|c| c.hand_count()).sum();
    if total_hand == 0 {
        return None;
    }
    let mut pick = (rng.next_u64() as u32) % total_hand;
    for (i, card) in cards.iter_mut().enumerate() {
        if pick < card.hand_count() {
            *card.hand_count_mut() -= 1;
            *card.discard_count_mut() += 1;
            return Some(i);
        }
        pick -= card.hand_count();
    }
    None
}
