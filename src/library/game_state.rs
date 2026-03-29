use super::action_log::ActionLog;
use super::types::{
    ActionEntry, ActionPayload, CardKind, ConcreteEffect, ConcreteEffectCost, EncounterKind,
    EncounterOutcome, EncounterState, HasDeckCounts,
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

fn roll_costs(
    rng: &mut rand_pcg::Lcg64Xsh32,
    costs: &[super::types::CardEffectCost],
    rolled_value: i64,
) -> Vec<ConcreteEffectCost> {
    costs
        .iter()
        .map(|c| {
            let rolled = roll_range_u32(rng, c.min_percent, c.max_percent);
            let amount = if c.is_absolute {
                rolled
            } else {
                (rolled_value.unsigned_abs() * rolled as u64 / 100) as u32
            };
            ConcreteEffectCost {
                token_type: c.token_type.clone(),
                amount,
            }
        })
        .collect()
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
            let costs = roll_costs(rng, &costs, value);
            (value, costs, Some(r_cap), Some(r_gain))
        }
        Some(super::types::CardEffectKind::LoseTokens {
            min, max, costs, ..
        }) => {
            let value = roll_range(rng, min, max);
            let costs = roll_costs(rng, &costs, value);
            (value, costs, None, None)
        }
        Some(super::types::CardEffectKind::Insight { min, max }) => {
            let value = roll_range(rng, min, max);
            (value, vec![], None, None)
        }
        Some(super::types::CardEffectKind::WoodcuttingChop {
            min_value,
            max_value,
            costs,
            ..
        }) => {
            let value = roll_range(rng, min_value as i64, max_value as i64);
            let costs = roll_costs(rng, &costs, value);
            (value, costs, None, None)
        }
        Some(super::types::CardEffectKind::HerbalismMatch { costs, .. }) => {
            let costs = roll_costs(rng, &costs, 0);
            (0, costs, None, None)
        }
        Some(super::types::CardEffectKind::FishingValue { min, max, costs }) => {
            let value = roll_range(rng, min, max);
            let costs = roll_costs(rng, &costs, value);
            (value, costs, None, None)
        }
        Some(super::types::CardEffectKind::CraftingReduction {
            min, max, costs, ..
        }) => {
            let value = roll_range(rng, min, max);
            let costs = roll_costs(rng, &costs, value);
            (value, costs, None, None)
        }
        Some(super::types::CardEffectKind::ResearchProbe { costs, .. }) => {
            let costs = roll_costs(rng, &costs, 0);
            (0, costs, None, None)
        }
        Some(super::types::CardEffectKind::ResearchInterference {
            kind: super::types::ResearchInterferenceKind::ReduceYield { min, max },
        }) => {
            let value = roll_range(rng, min, max);
            (value, vec![], None, None)
        }
        Some(super::types::CardEffectKind::ResearchInterference {
            kind:
                super::types::ResearchInterferenceKind::InsightTax {
                    min_percent,
                    max_percent,
                },
        }) => {
            let value = roll_range(rng, min_percent as i64, max_percent as i64);
            (value, vec![], None, None)
        }
        Some(super::types::CardEffectKind::ResearchInterference { .. }) => (0, vec![], None, None),
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

fn best_costs(
    costs: &[super::types::CardEffectCost],
    rolled_value: i64,
) -> Vec<ConcreteEffectCost> {
    costs
        .iter()
        .map(|c| {
            let amount = if c.is_absolute {
                c.max_percent
            } else {
                (rolled_value.unsigned_abs() * c.max_percent as u64 / 100) as u32
            };
            ConcreteEffectCost {
                token_type: c.token_type.clone(),
                amount,
            }
        })
        .collect()
}

/// Like `roll_concrete_effect` but always picks the maximum values for all ranges.
/// Used by milestones to create the "best version" of a card effect.
pub(crate) fn roll_best_concrete_effect(effect_id: usize, library: &Library) -> ConcreteEffect {
    let kind = library.resolve_effect(effect_id);
    let (rolled_value, rolled_costs, rolled_cap, rolled_gain_percent) = match kind {
        Some(super::types::CardEffectKind::GainTokens {
            cap_max,
            gain_max_percent,
            costs,
            ..
        }) => {
            let r_cap = cap_max;
            let r_gain = gain_max_percent;
            let value = r_cap * r_gain as i64 / 100;
            let costs = best_costs(&costs, value);
            (value, costs, Some(r_cap), Some(r_gain))
        }
        Some(super::types::CardEffectKind::LoseTokens { max, costs, .. }) => {
            let costs = best_costs(&costs, max);
            (max, costs, None, None)
        }
        Some(super::types::CardEffectKind::Insight { max, .. }) => (max, vec![], None, None),
        Some(super::types::CardEffectKind::WoodcuttingChop {
            max_value, costs, ..
        }) => {
            let value = max_value as i64;
            let costs = best_costs(&costs, value);
            (value, costs, None, None)
        }
        Some(super::types::CardEffectKind::HerbalismMatch { costs, .. }) => {
            let costs = best_costs(&costs, 0);
            (0, costs, None, None)
        }
        Some(super::types::CardEffectKind::FishingValue { max, costs, .. }) => {
            let costs = best_costs(&costs, max);
            (max, costs, None, None)
        }
        Some(super::types::CardEffectKind::CraftingReduction { max, costs, .. }) => {
            let costs = best_costs(&costs, max);
            (max, costs, None, None)
        }
        Some(super::types::CardEffectKind::ResearchProbe { costs, .. }) => {
            let costs = best_costs(&costs, 0);
            (0, costs, None, None)
        }
        Some(super::types::CardEffectKind::ResearchInterference {
            kind: super::types::ResearchInterferenceKind::ReduceYield { max, .. },
        }) => (max, vec![], None, None),
        Some(super::types::CardEffectKind::ResearchInterference {
            kind: super::types::ResearchInterferenceKind::InsightTax { max_percent, .. },
        }) => (max_percent as i64, vec![], None, None),
        Some(super::types::CardEffectKind::ResearchInterference { .. }) => (0, vec![], None, None),
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

/// Minimal in-memory game state driven by the library's mutator API.
#[derive(Debug, Clone)]
pub struct GameState {
    pub action_log: std::sync::Arc<ActionLog>,
    pub token_balances: HashMap<super::types::Token, i64>,
    pub library: Library,
    pub game_rules: super::config::GameRulesConfig,
    pub current_encounter: Option<EncounterState>,
    pub encounter_phase: super::types::EncounterPhase,
    pub last_encounter_result: Option<EncounterOutcome>,
    pub encounter_results: Vec<EncounterOutcome>,
    pub current_research: Option<super::types::ResearchProject>,
    pub encounter_records: Vec<super::types::EncounterRecord>,
    pub encounter_start_tokens: HashMap<super::types::TokenType, i64>,
    /// EncounterKind of the most recently finished encounter, used to generate
    /// scouting choices. Set when an encounter finishes, cleared after scouting.
    pub last_encounter_kind: Option<super::types::EncounterKind>,
    /// Card IDs of scouting-generated encounter choices. Cleared when the player
    /// picks their next encounter so un-selected choices can be removed.
    pub pending_scouting_choice_ids: Vec<usize>,
    /// Set to `true` when a player death occurs, cleared after the next scouting
    /// generation uses it to produce easier encounters.
    pub last_death_occurred: bool,
}

impl GameState {
    pub fn new() -> Self {
        use rand::SeedableRng;
        let mut rng = rand_pcg::Lcg64Xsh32::from_entropy();
        Self::new_with_rng(&mut rng)
    }

    pub fn new_with_rng(rng: &mut rand_pcg::Lcg64Xsh32) -> Self {
        let balances = super::config_loader::load_token_balances();
        let game_rules = super::config_loader::load_game_rules();
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
            library: super::config_loader::load_library(rng),
            game_rules,
            current_encounter: None,
            encounter_phase: super::types::EncounterPhase::NoEncounter,
            last_encounter_result: None,
            encounter_results: Vec::new(),
            current_research: None,
            encounter_records: Vec::new(),
            encounter_start_tokens: HashMap::new(),
            last_encounter_kind: None,
            pending_scouting_choice_ids: Vec::new(),
            last_death_occurred: false,
        }
    }

    /// Create a GameState from custom JSON configuration strings.
    ///
    /// `tokens_json` follows the `TokensConfig` format.
    /// `card_configs` is a slice of `(prefix, json_string)` pairs following
    /// the `DisciplineConfig` format — shared effects should come first.
    pub fn new_from_json(
        rng: &mut rand_pcg::Lcg64Xsh32,
        tokens_json: &str,
        card_configs: &[(&str, &str)],
    ) -> Self {
        let balances = super::config_loader::load_token_balances_from_json(tokens_json);
        let library = super::config_loader::load_library_from_json_configs(rng, card_configs);
        let game_rules = super::config_loader::load_game_rules();
        Self {
            action_log: std::sync::Arc::new(ActionLog::new()),
            token_balances: balances,
            library,
            game_rules,
            current_encounter: None,
            encounter_phase: super::types::EncounterPhase::NoEncounter,
            last_encounter_result: None,
            encounter_results: Vec::new(),
            current_research: None,
            encounter_records: Vec::new(),
            encounter_start_tokens: HashMap::new(),
            last_encounter_kind: None,
            pending_scouting_choice_ids: Vec::new(),
            last_death_occurred: false,
        }
    }

    /// Append an action to the action log with optional metadata; returns the appended entry.
    pub fn append_action(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
        self.action_log.append(action_type, payload)
    }

    /// Snapshot current token balances for metrics tracking at encounter start.
    pub fn snapshot_encounter_start_tokens(&mut self) {
        self.encounter_start_tokens = self
            .token_balances
            .iter()
            .map(|(token, &val)| (token.token_type.clone(), val))
            .collect();
    }

    /// Record a completed encounter for metrics. Call this before clearing current_encounter.
    pub(crate) fn record_encounter_finish(
        &mut self,
        discipline: super::types::Discipline,
        outcome: EncounterOutcome,
        rounds: u64,
    ) {
        let tokens_at_end: HashMap<super::types::TokenType, i64> = self
            .token_balances
            .iter()
            .map(|(token, &val)| (token.token_type.clone(), val))
            .collect();
        self.encounter_records.push(super::types::EncounterRecord {
            discipline,
            outcome: outcome.clone(),
            rounds,
            tokens_at_start: self.encounter_start_tokens.clone(),
            tokens_at_end,
        });
        self.last_encounter_result = Some(outcome.clone());
        self.encounter_results.push(outcome);
    }

    /// Capture the EncounterKind from the current encounter's library card
    /// before the encounter is cleared. Called by discipline finish functions.
    pub(crate) fn capture_last_encounter_kind(&mut self) {
        let enc_card_id = match &self.current_encounter {
            Some(enc) => enc.encounter_card_id(),
            None => return,
        };
        if let Some(card) = self.library.get(enc_card_id) {
            if let super::types::CardKind::Encounter { encounter_kind } = &card.kind {
                self.last_encounter_kind = Some(encounter_kind.clone());
            }
        }
    }

    /// Check if player can pay all costs on a card's effects. Deducts costs if affordable.
    /// Encounter-scoped token costs (e.g. RestToken) are filtered out — they are
    /// handled by the encounter state, not by persistent token_balances.
    pub(crate) fn check_and_deduct_costs(
        effects: &[ConcreteEffect],
        token_balances: &mut HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        let all_costs = Self::extract_gathering_costs_from_effects(effects);
        let player_costs: Vec<_> = all_costs
            .into_iter()
            .filter(|c| !c.token_type.is_encounter_scoped())
            .collect();
        Self::check_and_deduct_gathering_costs(&player_costs, token_balances)
    }

    /// Check if player can afford all costs without deducting. Used for pre-validation.
    /// Encounter-scoped token costs are filtered out.
    pub fn preview_costs(
        effects: &[ConcreteEffect],
        token_balances: &HashMap<super::types::Token, i64>,
    ) -> Result<(), String> {
        let all_costs = Self::extract_gathering_costs_from_effects(effects);
        let player_costs: Vec<_> = all_costs
            .into_iter()
            .filter(|c| !c.token_type.is_encounter_scoped())
            .collect();
        Self::preview_gathering_costs(&player_costs, token_balances)
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

    /// Extract gathering costs from ConcreteEffects' rolled_costs.
    /// Costs are pre-computed as absolute amounts at roll time.
    pub(crate) fn extract_gathering_costs_from_effects(
        effects: &[super::types::ConcreteEffect],
    ) -> Vec<super::types::TokenAmount> {
        let mut costs = Vec::new();
        for effect in effects {
            for cost in &effect.rolled_costs {
                let amount = cost.amount as i64;
                if amount > 0 {
                    costs.push(super::types::TokenAmount {
                        token_type: cost.token_type.clone(),
                        amount,
                        cap: None,
                    });
                }
            }
        }
        costs
    }

    /// Extract total rest token cost from effects' rolled_costs.
    pub(crate) fn extract_rest_token_cost(effects: &[super::types::ConcreteEffect]) -> i64 {
        let costs = Self::extract_gathering_costs_from_effects(effects);
        costs
            .iter()
            .filter(|c| c.token_type == super::types::TokenType::RestToken)
            .map(|c| c.amount)
            .sum()
    }

    /// Check if all gathering hand cards (effects-based) are unpayable.
    pub(crate) fn all_effects_hand_cards_unpayable(
        &self,
        effects_extractor: impl Fn(
            &super::types::CardKind,
        ) -> Option<&Vec<super::types::ConcreteEffect>>,
    ) -> bool {
        let hand_cards: Vec<_> = self
            .library
            .cards
            .iter()
            .filter(|c| c.counts.hand > 0 && effects_extractor(&c.kind).is_some())
            .collect();
        if hand_cards.is_empty() {
            return false;
        }
        hand_cards.iter().all(|card| {
            let effects = effects_extractor(&card.kind).unwrap();
            let costs = Self::extract_gathering_costs_from_effects(effects);
            let (pre_play_costs, _) = super::types::split_token_amounts(&costs);
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

    /// Check if the total card count (library + deck + hand + discard) for a given
    /// card kind has reached the MaxDeck token limit. Returns true when the limit is
    /// reached and no more cards of this kind should be added.
    pub(crate) fn decksize_reached(&self, kind: &super::types::CardKind) -> bool {
        if let Some(token) = super::types::TokenType::max_deck_token_for_kind(kind) {
            let max_deck = super::types::token_balance_by_type(&self.token_balances, &token);
            let kind_filter = super::types::CardKind::kind_matcher(kind);
            let current_total: u32 = self
                .library
                .cards
                .iter()
                .filter(|c| kind_filter(&c.kind))
                .map(|c| c.counts.library + c.counts.deck + c.counts.hand + c.counts.discard)
                .sum();
            current_total as i64 >= max_deck
        } else {
            false
        }
    }

    /// Abort a non-combat encounter: mark as lost, transition to Scouting.
    pub fn abort_encounter(&mut self) {
        let (discipline, rounds) = match &self.current_encounter {
            Some(enc) => (enc.discipline(), enc.round()),
            None => (super::types::Discipline::Combat, 0),
        };
        self.record_encounter_finish(discipline, EncounterOutcome::PlayerLost, rounds);
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
        self.token_balances
            .insert(health_key, self.game_rules.general.death_reset_health);
        self.token_balances.insert(
            super::types::Token::persistent(super::types::TokenType::Stamina),
            self.game_rules.general.death_reset_stamina,
        );

        // Increment player deaths counter
        let deaths_key = super::types::Token::persistent(super::types::TokenType::PlayerDeaths);
        let deaths = self.token_balances.entry(deaths_key).or_insert(0);
        *deaths += 1;

        // Signal scouting to generate easier encounters next time
        self.last_death_occurred = true;
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
                    gs.encounter_records.clear();
                    gs.encounter_start_tokens.clear();
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
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Research { .. },
                                } => {
                                    let _ = gs.start_research_encounter(card_id);
                                }
                                CardKind::Encounter {
                                    encounter_kind: EncounterKind::Milestone { .. },
                                } => {
                                    let _ = gs.start_milestone_encounter(card_id, &mut rng);
                                }
                                _ => {
                                    let _ = gs.start_combat(card_id, &mut rng);
                                }
                            }
                        }
                        gs.snapshot_encounter_start_tokens();
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
                        Some(EncounterState::Research(_)) => {
                            // Research encounters do not support card play
                        }
                        Some(EncounterState::Milestone(_)) => {
                            let _ = gs.resolve_milestone_play_card(*card_id, &mut rng);
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
                    } else if matches!(&gs.current_encounter, Some(EncounterState::Research(_))) {
                        gs.abort_research_encounter();
                    } else if matches!(&gs.current_encounter, Some(EncounterState::Milestone(_))) {
                        gs.abort_milestone_encounter();
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
                    Some(EncounterState::Research(_)) => {
                        let _ = gs.conclude_research_encounter();
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
                ActionPayload::ResearchChooseProject {
                    discipline,
                    tier_count,
                } => {
                    let _ = gs.research_choose_project(discipline.clone(), *tier_count, &mut rng);
                }
                ActionPayload::ResearchSelectCandidate { candidate_index } => {
                    let _ = gs.research_select_candidate(*candidate_index);
                }
                ActionPayload::ResearchProgress { amount } => {
                    let _ = gs.research_progress(*amount, &mut rng);
                }
                ActionPayload::ResearchPlayHand { card_ids } => {
                    let _ = gs.research_play_hand(card_ids.clone(), &mut rng);
                }
                ActionPayload::ResearchConcludeExperiment => {
                    let _ = gs.research_conclude_experiment(&mut rng);
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
