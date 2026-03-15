use std::collections::HashMap;

use rand::seq::SliceRandom;
use rand::Rng;

use crate::library::types::{
    CardCounts, CardKind, CombatantDef, ConcreteEffect, DeckCounts, EncounterKind, EnemyCardDef,
    EnemyCraftingCard, FishCard, InterferenceCard, OreCard, PlantCard, PlantCharacteristic, Token,
};
use crate::library::Library;

pub(crate) fn generate_scouting_choices(
    lib: &mut Library,
    rng: &mut rand_pcg::Lcg64Xsh32,
    source: &EncounterKind,
) -> Vec<usize> {
    let deltas = sample_difficulty_deltas(rng);
    deltas
        .iter()
        .map(|&delta| {
            let mutated = mutate_encounter_kind(rng, source, delta);
            let counts = CardCounts {
                library: 0,
                deck: 0,
                hand: 1,
                discard: 0,
            };
            lib.add_card(
                CardKind::Encounter {
                    encounter_kind: mutated,
                },
                counts,
                rng,
                vec![],
            )
        })
        .collect()
}

fn sample_difficulty_deltas(rng: &mut rand_pcg::Lcg64Xsh32) -> [f64; 3] {
    const MIN: f64 = -0.15;
    const MAX: f64 = 0.30;
    const MIN_SEP: f64 = 0.10;

    loop {
        let mut deltas = [
            rng.gen_range(MIN..=MAX),
            rng.gen_range(MIN..=MAX),
            rng.gen_range(MIN..=MAX),
        ];
        deltas.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if (deltas[1] - deltas[0]) >= MIN_SEP && (deltas[2] - deltas[1]) >= MIN_SEP {
            deltas.shuffle(rng);
            return deltas;
        }
    }
}

fn mutate_encounter_kind(
    rng: &mut rand_pcg::Lcg64Xsh32,
    source: &EncounterKind,
    delta: f64,
) -> EncounterKind {
    let factor = 1.0 + delta;
    match source {
        EncounterKind::Combat { combatant_def } => {
            let mut def = combatant_def.clone();
            def.initial_tokens = scale_token_map_u64(&def.initial_tokens, factor);
            mutate_combat_decks(rng, &mut def, delta);
            EncounterKind::Combat { combatant_def: def }
        }
        EncounterKind::Mining { mining_def } => {
            let mut def = mining_def.clone();
            def.initial_light_level = scale_i64(def.initial_light_level, factor);
            mutate_deck_generic(rng, &mut def.ore_deck, delta);
            EncounterKind::Mining { mining_def: def }
        }
        EncounterKind::Fishing { fishing_def } => {
            let mut def = fishing_def.clone();
            def.valid_range_min = scale_i64(def.valid_range_min, factor);
            def.valid_range_max = scale_i64(def.valid_range_max, factor);
            def.max_turns = scale_u32(def.max_turns, factor);
            def.win_turns_needed = scale_u32(def.win_turns_needed, factor);
            def.rewards = scale_token_map_i64(&def.rewards, factor);
            mutate_fish_deck(rng, &mut def.fish_deck, delta);
            EncounterKind::Fishing { fishing_def: def }
        }
        EncounterKind::Herbalism { herbalism_def } => {
            let mut def = herbalism_def.clone();
            def.rewards = scale_token_map_i64(&def.rewards, factor);
            mutate_plant_deck(rng, &mut def.plant_hand, delta);
            EncounterKind::Herbalism { herbalism_def: def }
        }
        EncounterKind::Woodcutting { woodcutting_def } => {
            let mut def = woodcutting_def.clone();
            def.max_plays = scale_u32(def.max_plays, factor);
            def.base_rewards = scale_token_map_i64(&def.base_rewards, factor);
            EncounterKind::Woodcutting {
                woodcutting_def: def,
            }
        }
        EncounterKind::Crafting { crafting_def } => {
            let mut def = crafting_def.clone();
            def.initial_crafting_tokens = scale_i64(def.initial_crafting_tokens, factor);
            mutate_deck_generic(rng, &mut def.enemy_crafting_deck, delta);
            EncounterKind::Crafting { crafting_def: def }
        }
        EncounterKind::Research { research_def } => {
            let mut def = research_def.clone();
            def.base_insight_cost = scale_i64(def.base_insight_cost, factor);
            def.position_match_yield = scale_i64(def.position_match_yield, factor);
            def.type_match_yield = scale_i64(def.type_match_yield, factor);
            mutate_deck_generic(rng, &mut def.interference_deck, delta);
            EncounterKind::Research { research_def: def }
        }
        EncounterKind::Rest { rest_def } => {
            let mut def = rest_def.clone();
            def.rest_token_min = scale_i64(def.rest_token_min, factor);
            def.rest_token_max = scale_i64(def.rest_token_max, factor);
            EncounterKind::Rest { rest_def: def }
        }
        EncounterKind::Milestone { milestone_def } => {
            let mut def = milestone_def.clone();
            let inner = mutate_encounter_kind(rng, &def.inner_encounter_kind, delta);
            def.inner_encounter_kind = Box::new(inner);
            EncounterKind::Milestone { milestone_def: def }
        }
    }
}

// ---------------------------------------------------------------------------
// Scaling helpers
// ---------------------------------------------------------------------------

fn scale_i64(val: i64, factor: f64) -> i64 {
    (val as f64 * factor).round() as i64
}

fn scale_u32(val: u32, factor: f64) -> u32 {
    ((val as f64 * factor).round() as u32).max(1)
}

fn scale_u64(val: u64, factor: f64) -> u64 {
    ((val as f64 * factor).round() as u64).max(1)
}

fn scale_token_map_i64(map: &HashMap<Token, i64>, factor: f64) -> HashMap<Token, i64> {
    map.iter()
        .map(|(k, &v)| (k.clone(), scale_i64(v, factor)))
        .collect()
}

fn scale_token_map_u64(map: &HashMap<Token, u64>, factor: f64) -> HashMap<Token, u64> {
    map.iter()
        .map(|(k, &v)| (k.clone(), scale_u64(v, factor)))
        .collect()
}

fn scale_effects(effects: &mut [ConcreteEffect], factor: f64) {
    for eff in effects.iter_mut() {
        eff.rolled_value = scale_i64(eff.rolled_value, factor);
        if let Some(cap) = eff.rolled_cap.as_mut() {
            *cap = scale_i64(*cap, factor);
        }
    }
}

// ---------------------------------------------------------------------------
// Trait to abstract over different card types that share effects + DeckCounts
// ---------------------------------------------------------------------------

trait MutableDeckEntry {
    fn effects_mut(&mut self) -> &mut Vec<ConcreteEffect>;
    fn counts_mut(&mut self) -> &mut DeckCounts;
    fn counts(&self) -> &DeckCounts;
}

impl MutableDeckEntry for EnemyCardDef {
    fn effects_mut(&mut self) -> &mut Vec<ConcreteEffect> {
        &mut self.effects
    }
    fn counts_mut(&mut self) -> &mut DeckCounts {
        &mut self.counts
    }
    fn counts(&self) -> &DeckCounts {
        &self.counts
    }
}

impl MutableDeckEntry for OreCard {
    fn effects_mut(&mut self) -> &mut Vec<ConcreteEffect> {
        &mut self.effects
    }
    fn counts_mut(&mut self) -> &mut DeckCounts {
        &mut self.counts
    }
    fn counts(&self) -> &DeckCounts {
        &self.counts
    }
}

impl MutableDeckEntry for EnemyCraftingCard {
    fn effects_mut(&mut self) -> &mut Vec<ConcreteEffect> {
        &mut self.effects
    }
    fn counts_mut(&mut self) -> &mut DeckCounts {
        &mut self.counts
    }
    fn counts(&self) -> &DeckCounts {
        &self.counts
    }
}

impl MutableDeckEntry for InterferenceCard {
    fn effects_mut(&mut self) -> &mut Vec<ConcreteEffect> {
        &mut self.effects
    }
    fn counts_mut(&mut self) -> &mut DeckCounts {
        &mut self.counts
    }
    fn counts(&self) -> &DeckCounts {
        &self.counts
    }
}

fn total_copies(c: &DeckCounts) -> u32 {
    c.deck + c.hand + c.discard
}

fn largest_pool_field(c: &DeckCounts) -> Pool {
    if c.deck >= c.hand && c.deck >= c.discard {
        Pool::Deck
    } else if c.hand >= c.discard {
        Pool::Hand
    } else {
        Pool::Discard
    }
}

enum Pool {
    Deck,
    Hand,
    Discard,
}

fn decrement_largest(c: &mut DeckCounts) {
    match largest_pool_field(c) {
        Pool::Deck => c.deck = c.deck.saturating_sub(1),
        Pool::Hand => c.hand = c.hand.saturating_sub(1),
        Pool::Discard => c.discard = c.discard.saturating_sub(1),
    }
}

// ---------------------------------------------------------------------------
// Generic deck mutation (for decks whose entries implement MutableDeckEntry)
// ---------------------------------------------------------------------------

fn mutate_deck_generic<T: MutableDeckEntry + Clone>(
    rng: &mut rand_pcg::Lcg64Xsh32,
    deck: &mut [T],
    delta: f64,
) {
    if deck.is_empty() {
        return;
    }
    let factor = 1.0 + delta;
    let num_to_mutate = compute_num_to_mutate(deck.len());
    let indices = pick_random_indices(rng, deck.len(), num_to_mutate);

    for &idx in &indices {
        let roll: f64 = rng.gen_range(0.0..1.0);
        if roll < 0.50 {
            scale_effects(deck[idx].effects_mut(), factor);
        } else if roll < 0.80 {
            redistribute_copies_generic(rng, deck, idx);
        } else {
            swap_tier_generic(rng, deck, idx);
        }
    }
}

fn redistribute_copies_generic<T: MutableDeckEntry>(
    rng: &mut rand_pcg::Lcg64Xsh32,
    deck: &mut [T],
    target_idx: usize,
) {
    if deck.len() < 2 {
        return;
    }
    let donor_idx = pick_other_index(rng, deck.len(), target_idx);
    let donor_total = total_copies(deck[donor_idx].counts());
    if donor_total == 0 {
        return;
    }
    decrement_largest(deck[donor_idx].counts_mut());
    deck[target_idx].counts_mut().hand += 1;
}

fn swap_tier_generic<T: MutableDeckEntry + Clone>(
    rng: &mut rand_pcg::Lcg64Xsh32,
    deck: &mut [T],
    target_idx: usize,
) {
    if deck.len() < 2 {
        return;
    }
    let source_idx = pick_other_index(rng, deck.len(), target_idx);
    let source_effects = deck[source_idx].effects_mut().clone();
    *deck[target_idx].effects_mut() = source_effects;
}

// ---------------------------------------------------------------------------
// Combat-specific: merge all three decks, mutate, then split back
// ---------------------------------------------------------------------------

fn mutate_combat_decks(rng: &mut rand_pcg::Lcg64Xsh32, def: &mut CombatantDef, delta: f64) {
    let atk_len = def.attack_deck.len();
    let def_len = def.defence_deck.len();

    let mut combined: Vec<EnemyCardDef> =
        Vec::with_capacity(atk_len + def_len + def.resource_deck.len());
    combined.append(&mut def.attack_deck);
    combined.append(&mut def.defence_deck);
    combined.append(&mut def.resource_deck);

    mutate_deck_generic(rng, &mut combined, delta);

    def.resource_deck = combined.split_off(atk_len + def_len);
    def.defence_deck = combined.split_off(atk_len);
    def.attack_deck = combined;
}

// ---------------------------------------------------------------------------
// Fish-deck mutation: also scales FishCard.value on ScaleValues
// ---------------------------------------------------------------------------

fn mutate_fish_deck(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [FishCard], delta: f64) {
    if deck.is_empty() {
        return;
    }
    let factor = 1.0 + delta;
    let num_to_mutate = compute_num_to_mutate(deck.len());
    let indices = pick_random_indices(rng, deck.len(), num_to_mutate);

    for &idx in &indices {
        let roll: f64 = rng.gen_range(0.0..1.0);
        if roll < 0.50 {
            deck[idx].value = scale_i64(deck[idx].value, factor);
            scale_effects(&mut deck[idx].effects, factor);
        } else if roll < 0.80 {
            redistribute_copies_fish(rng, deck, idx);
        } else {
            swap_tier_fish(rng, deck, idx);
        }
    }
}

fn redistribute_copies_fish(
    rng: &mut rand_pcg::Lcg64Xsh32,
    deck: &mut [FishCard],
    target_idx: usize,
) {
    if deck.len() < 2 {
        return;
    }
    let donor_idx = pick_other_index(rng, deck.len(), target_idx);
    let donor_total = total_copies(&deck[donor_idx].counts);
    if donor_total == 0 {
        return;
    }
    decrement_largest(&mut deck[donor_idx].counts);
    deck[target_idx].counts.hand += 1;
}

fn swap_tier_fish(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [FishCard], target_idx: usize) {
    if deck.len() < 2 {
        return;
    }
    let source_idx = pick_other_index(rng, deck.len(), target_idx);
    let source_effects = deck[source_idx].effects.clone();
    let source_value = deck[source_idx].value;
    deck[target_idx].effects = source_effects;
    deck[target_idx].value = source_value;
}

// ---------------------------------------------------------------------------
// Plant-deck mutation: mutate characteristics instead of scaling effects
// ---------------------------------------------------------------------------

const ALL_CHARACTERISTICS: [PlantCharacteristic; 5] = [
    PlantCharacteristic::Fragile,
    PlantCharacteristic::Thorny,
    PlantCharacteristic::Aromatic,
    PlantCharacteristic::Bitter,
    PlantCharacteristic::Luminous,
];

fn mutate_plant_deck(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [PlantCard], _delta: f64) {
    if deck.is_empty() {
        return;
    }
    let num_to_mutate = compute_num_to_mutate(deck.len());
    let indices = pick_random_indices(rng, deck.len(), num_to_mutate);

    for &idx in &indices {
        let roll: f64 = rng.gen_range(0.0..1.0);
        if roll < 0.50 {
            mutate_characteristics(rng, &mut deck[idx].characteristics);
        } else if roll < 0.80 {
            redistribute_copies_plant(rng, deck, idx);
        } else {
            swap_tier_plant(rng, deck, idx);
        }
    }
}

fn mutate_characteristics(rng: &mut rand_pcg::Lcg64Xsh32, chars: &mut Vec<PlantCharacteristic>) {
    if rng.gen_bool(0.5) && !chars.is_empty() {
        let remove_idx = rng.gen_range(0..chars.len());
        chars.remove(remove_idx);
    } else {
        let new_char = ALL_CHARACTERISTICS.choose(rng).unwrap().clone();
        if !chars.contains(&new_char) {
            chars.push(new_char);
        }
    }
}

fn redistribute_copies_plant(
    rng: &mut rand_pcg::Lcg64Xsh32,
    deck: &mut [PlantCard],
    target_idx: usize,
) {
    if deck.len() < 2 {
        return;
    }
    let donor_idx = pick_other_index(rng, deck.len(), target_idx);
    let donor_total = total_copies(&deck[donor_idx].counts);
    if donor_total == 0 {
        return;
    }
    decrement_largest(&mut deck[donor_idx].counts);
    deck[target_idx].counts.hand += 1;
}

fn swap_tier_plant(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [PlantCard], target_idx: usize) {
    if deck.len() < 2 {
        return;
    }
    let source_idx = pick_other_index(rng, deck.len(), target_idx);
    let source_effects = deck[source_idx].effects.clone();
    let source_chars = deck[source_idx].characteristics.clone();
    deck[target_idx].effects = source_effects;
    deck[target_idx].characteristics = source_chars;
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn compute_num_to_mutate(deck_len: usize) -> usize {
    ((deck_len as f64 * 0.20).ceil() as usize)
        .max(1)
        .min(deck_len)
}

fn pick_random_indices(rng: &mut rand_pcg::Lcg64Xsh32, len: usize, count: usize) -> Vec<usize> {
    let mut all: Vec<usize> = (0..len).collect();
    all.shuffle(rng);
    all.truncate(count);
    all
}

fn pick_other_index(rng: &mut rand_pcg::Lcg64Xsh32, len: usize, exclude: usize) -> usize {
    loop {
        let i = rng.gen_range(0..len);
        if i != exclude {
            return i;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn difficulty_deltas_have_minimum_separation() {
        let mut rng = rand_pcg::Lcg64Xsh32::seed_from_u64(42);
        for _ in 0..100 {
            let deltas = sample_difficulty_deltas(&mut rng);
            for &d in &deltas {
                assert!((-0.15..=0.30).contains(&d), "delta {d} out of range");
            }
            let mut sorted = deltas;
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            assert!(
                (sorted[1] - sorted[0]) >= 0.10 - 1e-9,
                "insufficient separation: {:?}",
                deltas
            );
            assert!(
                (sorted[2] - sorted[1]) >= 0.10 - 1e-9,
                "insufficient separation: {:?}",
                deltas
            );
        }
    }

    #[test]
    fn scale_helpers_work() {
        assert_eq!(scale_i64(100, 1.15), 115);
        assert_eq!(scale_i64(-10, 1.30), -13);
        assert_eq!(scale_u32(10, 0.85), 9);
        assert_eq!(scale_u32(1, 0.1), 1); // min 1
        assert_eq!(scale_u64(100, 1.30), 130);
    }
}
