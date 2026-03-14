use crate::library::types::{
    self, CardEffectKind, CardKind, CraftingCraftState, CraftingEncounterState, EncounterKind,
    EncounterOutcome, EncounterState,
};
use crate::library::GameState;
use rand::RngCore;

impl GameState {
    pub fn start_crafting_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        if self.current_encounter.is_some() {
            return Err("Already in an encounter".to_string());
        }

        let card = self
            .library
            .get(encounter_card_id)
            .ok_or("Encounter card not found")?
            .clone();
        let crafting_def = match &card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Crafting { crafting_def },
            } => crafting_def.clone(),
            _ => return Err("Card is not a crafting encounter".to_string()),
        };

        let mut enemy_deck = crafting_def.enemy_crafting_deck.clone();
        crate::library::game_state::deck_shuffle_hand(rng, &mut enemy_deck);

        self.current_encounter = Some(EncounterState::Crafting(CraftingEncounterState {
            round: 0,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            crafting_tokens: crafting_def.initial_crafting_tokens,
            enemy_crafting_deck: enemy_deck,
            active_craft: None,
        }));
        self.encounter_phase = types::EncounterPhase::InEncounter;

        // Draw crafting cards to hand
        self.draw_player_crafting_cards(5, rng);

        // Play the encounter card (move from hand to discard)
        let _ = self.library.play(encounter_card_id);

        Ok(())
    }

    /// Swap a card between deck/discard and library. Costs 1 crafting token.
    pub fn resolve_crafting_swap(&mut self, from_id: usize, to_id: usize) -> Result<(), String> {
        let crafting = match &mut self.current_encounter {
            Some(EncounterState::Crafting(c)) if c.outcome == EncounterOutcome::Undecided => c,
            _ => return Err("No active crafting encounter".to_string()),
        };

        if crafting.crafting_tokens < 1 {
            return Err("Not enough crafting tokens for swap".to_string());
        }

        // Validate from_id: must be a player card in deck or discard (not hand)
        let from_card = self
            .library
            .get(from_id)
            .ok_or("Source card not found")?
            .clone();
        if !is_player_card(&from_card.kind) {
            return Err("Can only swap player cards".to_string());
        }
        let from_in_deck_or_discard = from_card.counts.deck > 0 || from_card.counts.discard > 0;
        if !from_in_deck_or_discard {
            return Err("Source card must be in deck or discard pile".to_string());
        }

        // Validate to_id: must be a player card in library
        let to_card = self
            .library
            .get(to_id)
            .ok_or("Target card not found")?
            .clone();
        if !is_player_card(&to_card.kind) {
            return Err("Can only swap player cards".to_string());
        }
        if to_card.counts.library == 0 {
            return Err("Target card must be in library".to_string());
        }

        // Move from_card: deck/discard → library
        let from_card_mut = self
            .library
            .cards
            .get_mut(from_id)
            .ok_or("Card not found")?;
        if from_card_mut.counts.deck > 0 {
            from_card_mut.counts.deck -= 1;
        } else {
            from_card_mut.counts.discard -= 1;
        }
        from_card_mut.counts.library += 1;

        // Move to_card: library → deck
        let to_card_mut = self.library.cards.get_mut(to_id).ok_or("Card not found")?;
        to_card_mut.counts.library -= 1;
        to_card_mut.counts.deck += 1;

        // Deduct crafting token
        if let Some(EncounterState::Crafting(c)) = &mut self.current_encounter {
            c.crafting_tokens -= 1;
            if c.crafting_tokens <= 0 && c.active_craft.is_none() {
                self.finish_crafting_encounter(true);
            }
        }

        Ok(())
    }

    /// Add durability to a discipline. Costs 1 crafting token + wood or ore.
    pub fn resolve_crafting_add_durability(&mut self, discipline: &str) -> Result<(), String> {
        let crafting = match &self.current_encounter {
            Some(EncounterState::Crafting(c)) if c.outcome == EncounterOutcome::Undecided => c,
            _ => return Err("No active crafting encounter".to_string()),
        };

        if crafting.crafting_tokens < 1 {
            return Err("Not enough crafting tokens".to_string());
        }

        let (durability_token, cost_token, cost_amount) = match discipline {
            "Mining" => (
                types::TokenType::MiningDurability,
                types::TokenType::Ore,
                50,
            ),
            "Herbalism" => (
                types::TokenType::HerbalismDurability,
                types::TokenType::Lumber,
                50,
            ),
            "Woodcutting" => (
                types::TokenType::WoodcuttingDurability,
                types::TokenType::Lumber,
                50,
            ),
            "Fishing" => (
                types::TokenType::FishingDurability,
                types::TokenType::Ore,
                50,
            ),
            _ => {
                return Err(format!(
                    "Unknown discipline '{}'. Valid: Mining, Herbalism, Woodcutting, Fishing",
                    discipline
                ))
            }
        };

        // Check and deduct material cost
        let cost_key = types::Token::persistent(cost_token);
        let balance = self.token_balances.get(&cost_key).copied().unwrap_or(0);
        if balance < cost_amount {
            return Err(format!(
                "Not enough materials (need {}, have {})",
                cost_amount, balance
            ));
        }
        *self.token_balances.entry(cost_key).or_insert(0) -= cost_amount;

        // Grant durability
        let dur_key = types::Token::persistent(durability_token);
        *self.token_balances.entry(dur_key).or_insert(0) += 500;

        // Deduct crafting token
        if let Some(EncounterState::Crafting(c)) = &mut self.current_encounter {
            c.crafting_tokens -= 1;
            if c.crafting_tokens <= 0 && c.active_craft.is_none() {
                self.finish_crafting_encounter(true);
            }
        }

        Ok(())
    }

    /// Start crafting a copy of a card. Costs crafting tokens based on card quality.
    pub fn resolve_crafting_start_craft(&mut self, target_card_id: usize) -> Result<(), String> {
        let crafting = match &self.current_encounter {
            Some(EncounterState::Crafting(c)) if c.outcome == EncounterOutcome::Undecided => c,
            _ => return Err("No active crafting encounter".to_string()),
        };

        if crafting.active_craft.is_some() {
            return Err("A craft is already in progress".to_string());
        }

        let target_card = self
            .library
            .get(target_card_id)
            .ok_or("Target card not found")?
            .clone();
        if !is_player_card(&target_card.kind) {
            return Err("Can only craft copies of player cards".to_string());
        }

        // Crafting cost based on card quality — token cost is proportional
        let total_material_cost: i64 = target_card.crafting_cost.values().sum();
        let token_cost = ((total_material_cost / 100) + 1).min(crafting.crafting_tokens);
        let token_cost = token_cost.max(2); // Minimum 2 tokens to start a craft

        if crafting.crafting_tokens < token_cost {
            return Err(format!(
                "Not enough crafting tokens (need {}, have {})",
                token_cost, crafting.crafting_tokens
            ));
        }

        let craft_state = CraftingCraftState {
            target_card_id,
            original_costs: target_card.crafting_cost.clone(),
            current_costs: target_card.crafting_cost.clone(),
        };

        if let Some(EncounterState::Crafting(c)) = &mut self.current_encounter {
            c.crafting_tokens -= token_cost;
            c.active_craft = Some(craft_state);
        }

        Ok(())
    }

    /// Play a crafting card during the craft mini-game. Each turn costs 1 crafting token.
    pub fn resolve_crafting_play_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        {
            let crafting = match &self.current_encounter {
                Some(EncounterState::Crafting(c)) if c.outcome == EncounterOutcome::Undecided => c,
                _ => return Err("No active crafting encounter".to_string()),
            };

            if crafting.active_craft.is_none() {
                return Err("No active craft in progress".to_string());
            }

            if crafting.crafting_tokens < 1 {
                return Err("Not enough crafting tokens for this turn".to_string());
            }
        }

        let card = self.library.get(card_id).ok_or("Card not found")?.clone();
        let effects = match &card.kind {
            CardKind::Crafting { effects } => effects.clone(),
            _ => return Err("Card is not a crafting card".to_string()),
        };

        // Extract and deduct pre-play costs from rolled_costs on effects
        let costs = Self::extract_gathering_costs_from_effects(&effects);
        Self::check_and_deduct_gathering_costs(&costs, &mut self.token_balances)?;

        // Process all effects via library templates
        for effect in &effects {
            let kind = match self.library.resolve_effect(effect.effect_id) {
                Some(resolved) => resolved,
                None => continue,
            };
            match &kind {
                CardEffectKind::CraftingReduction { token_type, .. } => {
                    // Apply reduction to current craft costs (floor at 50% of original)
                    if let Some(EncounterState::Crafting(c)) = &mut self.current_encounter {
                        if let Some(ref mut craft) = c.active_craft {
                            if let Some(current) = craft.current_costs.get_mut(token_type) {
                                let original =
                                    craft.original_costs.get(token_type).copied().unwrap_or(0);
                                let floor = original / 2;
                                *current = (*current - effect.rolled_value).max(floor);
                            }
                        }
                    }
                }
                CardEffectKind::GainTokens { token_type, .. } => {
                    let entry = types::token_entry_by_type(&mut self.token_balances, token_type);
                    *entry += effect.rolled_value;
                }
                CardEffectKind::Insight { .. } => {
                    let insight_type =
                        types::TokenType::insight_for_discipline(&types::Discipline::Crafting);
                    let entry = types::token_entry_by_type(&mut self.token_balances, &insight_type);
                    *entry += effect.rolled_value;
                }
                // Costs handled via rolled_costs above
                _ => {}
            }
        }

        // Deduct 1 crafting token for the turn
        if let Some(EncounterState::Crafting(c)) = &mut self.current_encounter {
            c.crafting_tokens -= 1;
            c.round += 1;
        }

        // Play the card (hand → discard)
        let _ = self.library.play(card_id);

        // Enemy auto-plays: increase costs
        self.resolve_enemy_crafting_play(rng);

        // Draw replacement card
        self.draw_player_crafting_card(rng);

        // Check if crafting tokens are depleted — auto-conclude the craft
        if let Some(EncounterState::Crafting(c)) = &self.current_encounter {
            if c.crafting_tokens <= 0 {
                // Auto-conclude: check if player can pay
                self.auto_conclude_craft();
            }
        }

        Ok(())
    }

    /// Enemy plays a random crafting card, increasing material costs.
    fn resolve_enemy_crafting_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        // Extract effects from chosen card (mutable borrow scope)
        let effects = {
            let c = match &mut self.current_encounter {
                Some(EncounterState::Crafting(c)) if c.active_craft.is_some() => c,
                _ => return,
            };

            let total_in_deck: u32 = c
                .enemy_crafting_deck
                .iter()
                .map(|card| card.counts.deck)
                .sum();
            if total_in_deck == 0 {
                for card in &mut c.enemy_crafting_deck {
                    card.counts.deck += card.counts.discard;
                    card.counts.discard = 0;
                }
                return;
            }

            let pick = (rng.next_u64() as u32) % total_in_deck;
            let mut cumulative = 0u32;
            let mut chosen_idx = None;
            for (i, card) in c.enemy_crafting_deck.iter().enumerate() {
                cumulative += card.counts.deck;
                if pick < cumulative {
                    chosen_idx = Some(i);
                    break;
                }
            }

            match chosen_idx {
                Some(idx) => {
                    let fx = c.enemy_crafting_deck[idx].effects.clone();
                    c.enemy_crafting_deck[idx].counts.deck -= 1;
                    c.enemy_crafting_deck[idx].counts.discard += 1;
                    fx
                }
                None => return,
            }
        };

        // Resolve effects via library and apply increases to craft costs
        let mut resolved: Vec<(types::TokenType, i64)> = Vec::new();
        for effect in &effects {
            if let Some(types::CardEffectKind::GainTokens { token_type, .. }) =
                self.library.resolve_effect(effect.effect_id)
            {
                let grant_amount = match (effect.rolled_cap, effect.rolled_gain_percent) {
                    (Some(cap), Some(pct)) => cap * pct as i64 / 100,
                    _ => effect.rolled_value,
                };
                resolved.push((token_type.clone(), grant_amount));
            }
        }

        if let Some(EncounterState::Crafting(c)) = &mut self.current_encounter {
            if let Some(ref mut craft) = c.active_craft {
                for (token_type, amount) in resolved {
                    *craft.current_costs.entry(token_type).or_insert(0) += amount;
                }
            }
        }
    }

    /// Conclude the crafting encounter. If a craft is in progress, check if the
    /// player can pay the final costs. If yes, create a copy of the card in the library.
    pub fn conclude_crafting_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Crafting(c)) if c.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active crafting encounter to conclude".to_string()),
        };

        self.finish_active_craft();
        Ok(())
    }

    /// Auto-conclude when crafting tokens run out during a craft mini-game.
    fn auto_conclude_craft(&mut self) {
        if !matches!(&self.current_encounter, Some(EncounterState::Crafting(_))) {
            return;
        }

        self.finish_active_craft();
    }

    /// Shared logic for finishing an active craft (used by both conclude and auto-conclude).
    /// If there is an active craft, attempt to pay costs and add the crafted card.
    /// If no active craft, just finish the encounter as a win.
    fn finish_active_craft(&mut self) {
        let craft_info = match &self.current_encounter {
            Some(EncounterState::Crafting(c)) => c.active_craft.clone(),
            _ => {
                self.finish_crafting_encounter(true);
                return;
            }
        };

        let Some(craft) = craft_info else {
            self.finish_crafting_encounter(true);
            return;
        };

        let can_pay = craft.current_costs.iter().all(|(token_type, &cost)| {
            let key = types::Token::persistent(token_type.clone());
            self.token_balances.get(&key).copied().unwrap_or(0) >= cost
        });

        if !can_pay {
            self.finish_crafting_encounter(false);
            return;
        }

        for (token_type, cost) in &craft.current_costs {
            let key = types::Token::persistent(token_type.clone());
            *self.token_balances.entry(key).or_insert(0) -= cost;
        }

        if self
            .library
            .increment_library_count(craft.target_card_id, 1)
            .is_ok()
        {
            self.finish_crafting_encounter(true);
        } else {
            self.finish_crafting_encounter(false);
        }
    }

    /// Abort a crafting encounter. Always results in PlayerWon (no penalty).
    /// Abort a crafting encounter. Blocked when a craft mini-game is active —
    /// in that case only a successful craft can end the encounter as a win.
    pub fn abort_crafting_encounter(&mut self) -> Result<(), String> {
        if let Some(EncounterState::Crafting(c)) = &self.current_encounter {
            if c.active_craft.is_some() {
                return Err(
                    "Cannot abort while a craft is in progress. Complete or fail the craft first."
                        .to_string(),
                );
            }
        }
        let rounds = match &self.current_encounter {
            Some(EncounterState::Crafting(c)) => c.round,
            _ => 0,
        };
        self.record_encounter_finish(
            types::Discipline::Crafting,
            EncounterOutcome::PlayerWon,
            rounds,
        );
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
        Ok(())
    }

    pub(crate) fn finish_crafting_encounter(&mut self, is_win: bool) {
        let outcome = if is_win {
            EncounterOutcome::PlayerWon
        } else {
            EncounterOutcome::PlayerLost
        };
        let rounds = match &self.current_encounter {
            Some(EncounterState::Crafting(c)) => c.round,
            _ => 0,
        };
        self.record_encounter_finish(types::Discipline::Crafting, outcome, rounds);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }

    fn draw_player_crafting_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Crafting { .. }),
            rng,
            Some(types::TokenType::CraftingMaxHand),
        );
    }

    fn draw_player_crafting_cards(&mut self, count: u32, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            count,
            |k| matches!(k, CardKind::Crafting { .. }),
            rng,
            Some(types::TokenType::CraftingMaxHand),
        );
    }
}

fn is_player_card(kind: &CardKind) -> bool {
    matches!(
        kind,
        CardKind::Attack { .. }
            | CardKind::Defence { .. }
            | CardKind::Resource { .. }
            | CardKind::Mining { .. }
            | CardKind::Herbalism { .. }
            | CardKind::Woodcutting { .. }
            | CardKind::Fishing { .. }
            | CardKind::Rest { .. }
            | CardKind::Crafting { .. }
    )
}
