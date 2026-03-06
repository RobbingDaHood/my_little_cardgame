//! Minimal domain skeleton for Decks, Tokens, Library and ActionLog
//!
//! This file provides small, well-scoped domain primitives used by higher-level systems.

pub mod action_log;
pub(crate) mod disciplines;
mod endpoints;
pub mod game_state;
pub mod types;

pub use endpoints::{
    add_test_library_card, list_card_effects, list_library_cards,
    okapi_add_operation_for_list_card_effects_, CardEffectEntry, CardEffectsResponse,
};
pub use game_state::GameState;

use std::collections::HashMap;
use types::{CardCounts, CardKind, EncounterKind, LibraryCard};

/// Calculate the crafting cost for a card based on its effects.
/// Higher rolled values and more effects = higher cost.
/// Returns material costs spread across Ore, Plant, Lumber, Fish.
fn calculate_crafting_cost(kind: &CardKind) -> HashMap<types::TokenType, i64> {
    let mut total_power: i64 = 0;
    let num_effects: i64;

    match kind {
        CardKind::Attack { effects }
        | CardKind::Defence { effects }
        | CardKind::Resource { effects }
        | CardKind::Rest { effects, .. } => {
            num_effects = effects.len() as i64;
            for effect in effects {
                total_power += effect.rolled_value.abs();
            }
        }
        CardKind::Mining { mining_effect } => {
            num_effects = mining_effect.gains.len() as i64;
            for gain in &mining_effect.gains {
                total_power += gain.amount.abs();
            }
        }
        CardKind::Herbalism { herbalism_effect } => {
            num_effects = 1;
            for gain in &herbalism_effect.gains {
                total_power += gain.amount.abs();
            }
        }
        CardKind::Woodcutting { woodcutting_effect } => {
            num_effects = woodcutting_effect.chop_values.len() as i64;
            for &val in &woodcutting_effect.chop_values {
                total_power += val as i64;
            }
        }
        CardKind::Fishing { fishing_effect } => {
            num_effects = fishing_effect.values.len() as i64;
            for &val in &fishing_effect.values {
                total_power += val.abs();
            }
        }
        CardKind::Crafting { crafting_effect } => {
            num_effects = crafting_effect.reductions.len() as i64;
            for r in &crafting_effect.reductions {
                total_power += r.amount.abs();
            }
        }
        // Non-player cards have no crafting cost
        CardKind::Encounter { .. }
        | CardKind::PlayerCardEffect { .. }
        | CardKind::EnemyCardEffect { .. } => {
            return HashMap::new();
        }
    }

    if total_power == 0 {
        return HashMap::new();
    }

    // Superlinear scaling: base_cost = total_power * (1 + num_effects) / 4
    let base_cost = total_power * (1 + num_effects) / 4;
    let materials = [
        types::TokenType::Ore,
        types::TokenType::Plant,
        types::TokenType::Lumber,
        types::TokenType::Fish,
    ];
    let per_material = (base_cost / materials.len() as i64).max(1);

    let mut cost = HashMap::new();
    for mat in &materials {
        cost.insert(mat.clone(), per_material);
    }
    cost
}

/// The Library: canonical collection of all player-owned cards.
/// Index in the Vec = card ID. Per vision "card location model and counts".
#[derive(Debug, Clone)]
pub struct Library {
    pub cards: Vec<LibraryCard>,
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl Library {
    pub fn new() -> Self {
        Library { cards: Vec::new() }
    }

    /// Add a card to the library. Returns the card ID (index).
    /// Panics if a GainTokens effect has a cost token_type matching its gain token_type.
    pub fn add_card(&mut self, kind: CardKind, counts: CardCounts) -> usize {
        // Validate GainTokens: gain token_type must not match any cost token_type
        let effect_kind = match &kind {
            CardKind::PlayerCardEffect { kind: k, .. }
            | CardKind::EnemyCardEffect { kind: k, .. } => Some(k),
            _ => None,
        };
        if let Some(types::CardEffectKind::GainTokens {
            token_type, costs, ..
        }) = effect_kind
        {
            for cost in costs {
                assert!(
                    cost.token_type != *token_type,
                    "GainTokens cannot have a token_type ({:?}) matching its gain token_type ({:?})",
                    cost.token_type,
                    token_type
                );
            }
        }
        let id = self.cards.len();
        let crafting_cost = calculate_crafting_cost(&kind);
        self.cards.push(LibraryCard {
            kind,
            counts,
            crafting_cost,
        });
        id
    }

    /// Get a card by ID (index).
    pub fn get(&self, card_id: usize) -> Option<&LibraryCard> {
        self.cards.get(card_id)
    }

    /// Draw a card: move one copy from deck → hand.
    pub fn draw(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.deck == 0 {
            return Err(format!("Card {card_id} has no copies in deck"));
        }
        card.counts.deck -= 1;
        card.counts.hand += 1;
        Ok(())
    }

    /// Play/discard a card: move one copy from hand → discard.
    pub fn play(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.hand == 0 {
            return Err(format!("Card {card_id} has no copies in hand"));
        }
        card.counts.hand -= 1;
        card.counts.discard += 1;
        Ok(())
    }

    /// Return a card from discard → library.
    pub fn return_to_library(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.discard == 0 {
            return Err(format!("Card {card_id} has no copies in discard"));
        }
        card.counts.discard -= 1;
        card.counts.library += 1;
        Ok(())
    }

    /// Move copies from library → deck (adding cards to your deck).
    pub fn add_to_deck(&mut self, card_id: usize, count: u32) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.library < count {
            return Err(format!(
                "Card {card_id} has only {} copies in library, need {count}",
                card.counts.library
            ));
        }
        card.counts.library -= count;
        card.counts.deck += count;
        Ok(())
    }

    /// Resolve a card effect entry by ID, returning its kind.
    /// Only works for PlayerCardEffect and EnemyCardEffect entries.
    pub fn resolve_effect(&self, effect_id: usize) -> Option<types::CardEffectKind> {
        let card = self.cards.get(effect_id)?;
        match &card.kind {
            CardKind::PlayerCardEffect { kind, .. } | CardKind::EnemyCardEffect { kind, .. } => {
                Some(kind.clone())
            }
            _ => None,
        }
    }

    /// All cards currently on hand.
    pub fn hand_cards(&self) -> Vec<(usize, &LibraryCard)> {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, c)| c.counts.hand > 0)
            .collect()
    }

    /// All cards matching a predicate on CardKind.
    pub fn cards_matching<F>(&self, predicate: F) -> Vec<(usize, &LibraryCard)>
    where
        F: Fn(&CardKind) -> bool,
    {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, c)| predicate(&c.kind))
            .collect()
    }

    /// Return a card from discard → deck (recycle).
    pub fn return_to_deck(&mut self, card_id: usize) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(card_id)
            .ok_or_else(|| format!("Card {card_id} not found"))?;
        if card.counts.discard == 0 {
            return Err(format!("Card {card_id} has no copies in discard"));
        }
        card.counts.discard -= 1;
        card.counts.deck += 1;
        Ok(())
    }

    /// Encounter cards currently in the hand (visible/pickable).
    pub fn encounter_hand(&self) -> Vec<usize> {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
            .flat_map(|(id, c)| std::iter::repeat_n(id, c.counts.hand as usize))
            .collect()
    }

    /// Check if an encounter card is in the hand.
    pub fn encounter_contains(&self, card_id: usize) -> bool {
        self.cards
            .get(card_id)
            .is_some_and(|c| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
    }

    /// Draw encounter cards from deck to hand until hand reaches target_count.
    pub fn encounter_draw_to_hand(&mut self, target_count: usize) {
        let current_hand: usize = self
            .cards
            .iter()
            .filter(|c| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
            .map(|c| c.counts.hand as usize)
            .sum();
        let mut remaining = target_count.saturating_sub(current_hand);
        for card in &mut self.cards {
            if remaining == 0 {
                break;
            }
            if matches!(card.kind, CardKind::Encounter { .. }) && card.counts.deck > 0 {
                let to_move = (card.counts.deck as usize).min(remaining) as u32;
                card.counts.deck -= to_move;
                card.counts.hand += to_move;
                remaining -= to_move as usize;
            }
        }
    }

    /// Validate that all card effects reference valid CardEffect deck entries.
    pub fn validate_card_effects(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for (id, card) in self.cards.iter().enumerate() {
            match &card.kind {
                CardKind::Attack { effects }
                | CardKind::Defence { effects }
                | CardKind::Resource { effects }
                | CardKind::Rest { effects, .. } => {
                    for effect in effects {
                        match self.cards.get(effect.effect_id) {
                            Some(ref_card)
                                if matches!(ref_card.kind, CardKind::PlayerCardEffect { .. }) => {}
                            _ => errors.push(format!(
                                "Card {} has effect referencing invalid PlayerCardEffect {}",
                                id, effect.effect_id
                            )),
                        }
                    }
                }
                CardKind::Encounter {
                    encounter_kind: EncounterKind::Combat { combatant_def },
                } => {
                    for deck in [
                        &combatant_def.attack_deck,
                        &combatant_def.defence_deck,
                        &combatant_def.resource_deck,
                    ] {
                        for enemy_card in deck {
                            for effect in &enemy_card.effects {
                                match self.cards.get(effect.effect_id) {
                                    Some(ref_card)
                                        if matches!(
                                            ref_card.kind,
                                            CardKind::EnemyCardEffect { .. }
                                        ) => {}
                                    _ => errors.push(format!(
                                        "Enemy card in card {} has effect referencing invalid EnemyCardEffect {}",
                                        id, effect.effect_id
                                    )),
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
