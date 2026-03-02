use crate::library::types::{
    self, CardKind, EncounterKind, EncounterOutcome, EncounterState, MiningEncounterState,
};
use crate::library::GameState;

impl GameState {
    /// Initialize a mining gathering encounter from a Library Encounter card.
    pub fn start_mining_encounter(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let mining_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Mining { mining_def },
            } => mining_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a mining encounter",
                    encounter_card_id
                ))
            }
        };
        let mut ore_deck = mining_def.ore_deck.clone();
        Self::ore_shuffle_hand(rng, &mut ore_deck);
        let state = MiningEncounterState {
            round: 1,
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            ore_tokens: mining_def.initial_tokens,
            ore_deck,
            rewards: mining_def.rewards,
        };
        self.current_encounter = Some(EncounterState::Mining(state));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    /// Resolve a player mining card play against the current mining encounter.
    /// Applies ore damage, stores durability prevent, auto-resolves ore play,
    /// draws cards for both sides, and checks encounter end.
    pub fn resolve_player_mining_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(card_id)
            .ok_or_else(|| format!("Card {} not found in Library", card_id))?
            .clone();
        let mining_effect = match &lib_card.kind {
            CardKind::Mining { mining_effect } => mining_effect.clone(),
            _ => return Err("Cannot play a non-mining card in mining encounter".to_string()),
        };

        // Check and deduct pre-play costs (stamina etc. — reject card if unaffordable)
        Self::check_and_deduct_gathering_costs(&mining_effect.costs, &mut self.token_balances)?;

        // Apply gains
        for gain in &mining_effect.gains {
            let entry = types::token_entry_by_type(&mut self.token_balances, &gain.cost_type);
            *entry += gain.amount;
        }

        // Apply player mining card: damage ore
        let ore_defeated = {
            let mining = match &mut self.current_encounter {
                Some(EncounterState::Mining(m)) => m,
                _ => return Err("No active mining encounter".to_string()),
            };
            let ore_health_key = types::Token::persistent(types::TokenType::OreHealth);
            let ore_hp = mining.ore_tokens.entry(ore_health_key).or_insert(0);
            *ore_hp = (*ore_hp - mining_effect.ore_damage).max(0);
            *ore_hp <= 0
        };

        if ore_defeated {
            self.finish_mining_encounter(true);
            return Ok(());
        }

        // Auto-resolve ore play with the prevent value from the card just played
        self.resolve_ore_play(rng, mining_effect.durability_prevent);

        // Player draws a mining card
        self.draw_player_mining_card(rng);

        Ok(())
    }

    /// Ore plays a random card from hand, dealing durability damage minus prevent.
    /// Then draws a card from deck to hand.
    fn resolve_ore_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32, durability_prevent: i64) {
        use rand::RngCore;

        // Play ore card and extract damage info
        let (effective_damage, played) = {
            let mining = match &mut self.current_encounter {
                Some(EncounterState::Mining(m)) => m,
                _ => return,
            };
            let hand_indices: Vec<usize> = mining
                .ore_deck
                .iter()
                .enumerate()
                .filter(|(_, c)| c.counts.hand > 0)
                .map(|(i, _)| i)
                .collect();
            if hand_indices.is_empty() {
                return;
            }
            let pick_idx = (rng.next_u64() as usize) % hand_indices.len();
            let card_idx = hand_indices[pick_idx];
            mining.ore_deck[card_idx].counts.hand -= 1;
            mining.ore_deck[card_idx].counts.discard += 1;
            let raw_damage = mining.ore_deck[card_idx].durability_damage;
            let effective = (raw_damage - durability_prevent).max(0);
            mining.round += 1;
            (effective, true)
        };

        if !played {
            return;
        }

        // Apply durability damage to player
        let durability_key = types::Token::persistent(types::TokenType::MiningDurability);
        let durability = self
            .token_balances
            .entry(durability_key.clone())
            .or_insert(0);
        *durability = (*durability - effective_damage).max(0);

        // Ore draws a card
        if let Some(EncounterState::Mining(mining)) = &mut self.current_encounter {
            Self::ore_draw_random(rng, &mut mining.ore_deck);
        }

        // Check if player durability is depleted
        let durability = self
            .token_balances
            .get(&durability_key)
            .copied()
            .unwrap_or(0);
        if durability <= 0 {
            self.finish_mining_encounter(false);
        }
    }

    /// Finalize a mining encounter: grant rewards (win) or apply penalties (loss).
    fn finish_mining_encounter(&mut self, is_win: bool) {
        if is_win {
            let rewards = match &self.current_encounter {
                Some(EncounterState::Mining(m)) => m.rewards.clone(),
                _ => return,
            };
            for (token, amount) in &rewards {
                let entry = self.token_balances.entry(token.clone()).or_insert(0);
                *entry += amount;
            }
        }
        let outcome = if is_win {
            EncounterOutcome::PlayerWon
        } else {
            EncounterOutcome::PlayerLost
        };
        self.last_encounter_result = Some(outcome.clone());
        self.encounter_results.push(outcome);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }

    /// Draw one player mining card from deck to hand, recycling discard if needed.
    fn draw_player_mining_card(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) {
        self.draw_player_cards_of_kind(
            1,
            |k| matches!(k, CardKind::Mining { .. }),
            rng,
            Some(types::TokenType::MiningMaxHand),
        );
    }

    /// Shuffle ore hand: move all to deck, redraw to original hand size.
    fn ore_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::OreCard]) {
        let target_hand: u32 = deck.iter().map(|c| c.counts.hand).sum();
        for card in deck.iter_mut() {
            card.counts.deck += card.counts.hand;
            card.counts.hand = 0;
        }
        for _ in 0..target_hand {
            Self::ore_draw_random(rng, deck);
        }
    }

    /// Draw one random ore card from deck to hand, recycling discard if needed.
    fn ore_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [types::OreCard]) {
        use rand::RngCore;
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
        if total_deck == 0 {
            let total_discard: u32 = deck.iter().map(|c| c.counts.discard).sum();
            if total_discard == 0 {
                return;
            }
            for card in deck.iter_mut() {
                card.counts.deck += card.counts.discard;
                card.counts.discard = 0;
            }
        }
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
        if total_deck == 0 {
            return;
        }
        let mut pick = (rng.next_u64() as u32) % total_deck;
        for card in deck.iter_mut() {
            if pick < card.counts.deck {
                card.counts.deck -= 1;
                card.counts.hand += 1;
                return;
            }
            pick -= card.counts.deck;
        }
    }
}
