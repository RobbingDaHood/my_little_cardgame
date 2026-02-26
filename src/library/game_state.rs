use super::action_log::ActionLog;
use super::types::{ActionEntry, ActionPayload, CardCounts, CardKind, EncounterKind};
use super::Library;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

fn initialize_library() -> Library {
    let mut lib = Library::new();

    // ---- Player CardEffect deck entries ----

    // id 0: Player "deal 5 damage" effect
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                amount: -5,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 1: Player "grant 3 shield" effect
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                amount: 3,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 2: Player "grant 2 stamina" effect
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                amount: 2,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 3: Player "draw 1 attack, 1 defence, 2 resource" effect
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

    // ---- Enemy CardEffect deck entries ----

    // id 4: Enemy "deal 3 damage" effect
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnOpponent,
                token_type: super::types::TokenType::Health,
                amount: -3,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 5: Enemy "grant 2 shield" effect
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Shield,
                amount: 2,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 6: Enemy "grant 1 stamina" effect
    lib.add_card(
        CardKind::EnemyCardEffect {
            kind: super::types::CardEffectKind::ChangeTokens {
                target: super::types::EffectTarget::OnSelf,
                token_type: super::types::TokenType::Stamina,
                amount: 1,
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );

    // id 7: Enemy "draw 1 attack, 1 defence, 2 resource" effect
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

    // ---- Player action cards (reference CardEffect entries by id) ----

    // Attack card (id 8): deals 5 damage to opponent
    lib.add_card(
        CardKind::Attack {
            effect_ids: vec![0],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Defence card (id 9): grants 3 shield to self
    lib.add_card(
        CardKind::Defence {
            effect_ids: vec![1],
        },
        CardCounts {
            library: 0,
            deck: 15,
            hand: 5,
            discard: 0,
        },
    );

    // Resource card (id 10): grants 2 stamina to self, draws 2 cards
    lib.add_card(
        CardKind::Resource {
            effect_ids: vec![2, 3],
        },
        CardCounts {
            library: 0,
            deck: 35,
            hand: 5,
            discard: 0,
        },
    );

    // Combat encounter: Gnome (id 11)
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: super::types::EncounterKind::Combat {
                combatant_def: super::types::CombatantDef {
                    initial_tokens: HashMap::from([
                        (
                            super::types::Token::persistent(super::types::TokenType::Health),
                            20,
                        ),
                        (
                            super::types::Token::persistent(super::types::TokenType::MaxHealth),
                            20,
                        ),
                    ]),
                    attack_deck: vec![super::types::EnemyCardDef {
                        effect_ids: vec![4],
                        counts: super::types::EnemyCardCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
                    defence_deck: vec![super::types::EnemyCardDef {
                        effect_ids: vec![5],
                        counts: super::types::EnemyCardCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
                    resource_deck: vec![super::types::EnemyCardDef {
                        effect_ids: vec![6, 7],
                        counts: super::types::EnemyCardCounts {
                            deck: 0,
                            hand: 10,
                            discard: 0,
                        },
                    }],
                },
            },
        },
        CardCounts {
            library: 1,
            deck: 0,
            hand: 3,
            discard: 0,
        },
    );

    if let Err(errors) = lib.validate_card_effects() {
        panic!("Library card effect validation failed: {:?}", errors);
    }

    lib
}

/// Apply card effects to combat, modifying player tokens and combat snapshot.
/// Resolves effect IDs from library card effect entries.
/// Only processes ChangeTokens effects; DrawCards effects are handled separately.
fn apply_card_effects(
    effect_ids: &[usize],
    is_player: bool,
    player_tokens: &mut HashMap<super::types::Token, i64>,
    combat: &mut super::types::CombatState,
    library: &Library,
) {
    for &effect_id in effect_ids {
        let kind = match library.resolve_effect(effect_id) {
            Some(resolved) => resolved,
            None => continue,
        };

        let (target, token_type, amount) = match &kind {
            super::types::CardEffectKind::ChangeTokens {
                target,
                token_type,
                amount,
            } => (target, token_type, *amount),
            super::types::CardEffectKind::DrawCards { .. } => continue,
        };

        let target_tokens = match (target, is_player) {
            (super::types::EffectTarget::OnSelf, true)
            | (super::types::EffectTarget::OnOpponent, false) => &mut *player_tokens,
            (super::types::EffectTarget::OnOpponent, true)
            | (super::types::EffectTarget::OnSelf, false) => &mut combat.enemy_tokens,
        };

        if *token_type == super::types::TokenType::Health && amount < 0 {
            let damage = -amount;
            let dodge = target_tokens
                .get(&super::types::Token::dodge())
                .copied()
                .unwrap_or(0);
            let absorbed = dodge.min(damage);
            target_tokens.insert(super::types::Token::dodge(), (dodge - absorbed).max(0));
            let remaining_damage = damage - absorbed;
            if remaining_damage > 0 {
                let health = target_tokens
                    .entry(super::types::Token::persistent(
                        super::types::TokenType::Health,
                    ))
                    .or_insert(0);
                *health = (*health - remaining_damage).max(0);
            }
        } else {
            let entry = target_tokens
                .entry(super::types::Token::persistent(token_type.clone()))
                .or_insert(0);
            *entry = (*entry + amount).max(0);
        }
    }
}

/// Check if combat has ended (either side at 0 health).
fn check_combat_end(
    player_tokens: &HashMap<super::types::Token, i64>,
    combat: &mut super::types::CombatState,
) {
    let player_health = player_tokens
        .get(&super::types::Token::persistent(
            super::types::TokenType::Health,
        ))
        .copied()
        .unwrap_or(0);
    let enemy_health = combat
        .enemy_tokens
        .get(&super::types::Token::persistent(
            super::types::TokenType::Health,
        ))
        .copied()
        .unwrap_or(0);

    if enemy_health <= 0 || player_health <= 0 {
        combat.is_finished = true;
        combat.outcome = if enemy_health <= 0 && player_health > 0 {
            super::types::CombatOutcome::PlayerWon
        } else if player_health <= 0 && enemy_health > 0 {
            super::types::CombatOutcome::EnemyWon
        } else {
            super::types::CombatOutcome::PlayerWon // Draw defaults to player
        };
    }
}

/// Minimal in-memory game state driven by the library's mutator API.
#[derive(Debug, Clone)]
pub struct GameState {
    pub action_log: std::sync::Arc<ActionLog>,
    pub token_balances: HashMap<super::types::Token, i64>,
    pub library: Library,
    pub current_combat: Option<super::types::CombatState>,
    pub encounter_phase: super::types::EncounterPhase,
    pub last_combat_result: Option<super::types::CombatOutcome>,
    pub combat_results: Vec<super::types::CombatOutcome>,
}

impl GameState {
    pub fn new() -> Self {
        let mut balances = HashMap::new();
        for id in super::types::TokenType::all() {
            balances.insert(super::types::Token::persistent(id), 0i64);
        }
        // Default Foresight controls area deck hand size
        balances.insert(
            super::types::Token::persistent(super::types::TokenType::Foresight),
            3,
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
            library: initialize_library(),
            current_combat: None,
            encounter_phase: super::types::EncounterPhase::NoEncounter,
            last_combat_result: None,
            combat_results: Vec::new(),
        }
    }

    /// Append an action to the action log with optional metadata; returns the appended entry.
    pub fn append_action(&self, action_type: &str, payload: ActionPayload) -> ActionEntry {
        self.action_log.append(action_type, payload)
    }

    /// Initialize combat from a Library Encounter card.
    pub fn start_combat(
        &mut self,
        encounter_card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let lib_card = self
            .library
            .get(encounter_card_id)
            .ok_or_else(|| format!("Card {} not found in Library", encounter_card_id))?
            .clone();
        let combatant_def = match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Combat { combatant_def },
            } => combatant_def.clone(),
            _ => {
                return Err(format!(
                    "Card {} is not a combat encounter",
                    encounter_card_id
                ))
            }
        };
        let mut enemy_attack_deck = combatant_def.attack_deck.clone();
        let mut enemy_defence_deck = combatant_def.defence_deck.clone();
        let mut enemy_resource_deck = combatant_def.resource_deck.clone();
        Self::enemy_shuffle_hand(rng, &mut enemy_attack_deck);
        Self::enemy_shuffle_hand(rng, &mut enemy_defence_deck);
        Self::enemy_shuffle_hand(rng, &mut enemy_resource_deck);
        let snapshot = super::types::CombatState {
            round: 1,
            phase: super::types::CombatPhase::Defending,
            enemy_tokens: combatant_def
                .initial_tokens
                .iter()
                .map(|(k, v)| (k.clone(), *v as i64))
                .collect(),
            encounter_card_id: Some(encounter_card_id),
            is_finished: false,
            outcome: super::types::CombatOutcome::Undecided,
            enemy_attack_deck,
            enemy_defence_deck,
            enemy_resource_deck,
        };
        self.current_combat = Some(snapshot);
        self.encounter_phase = super::types::EncounterPhase::Combat;
        Ok(())
    }

    /// Resolve a player card play against the current combat snapshot.
    pub fn resolve_player_card(
        &mut self,
        card_id: usize,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
        let lib_card = self
            .library
            .get(card_id)
            .ok_or_else(|| format!("Card {} not found in Library", card_id))?
            .clone();
        let effect_ids = match &lib_card.kind {
            CardKind::Attack { effect_ids }
            | CardKind::Defence { effect_ids }
            | CardKind::Resource { effect_ids } => effect_ids.clone(),
            _ => return Err("Cannot play a non-action card".to_string()),
        };
        let (mut atk_draws, mut def_draws, mut res_draws) = (0u32, 0u32, 0u32);
        for &id in &effect_ids {
            if let Some(super::types::CardEffectKind::DrawCards {
                attack,
                defence,
                resource,
            }) = self.library.resolve_effect(id)
            {
                atk_draws += attack;
                def_draws += defence;
                res_draws += resource;
            }
        }
        apply_card_effects(
            &effect_ids,
            true,
            &mut self.token_balances,
            combat,
            &self.library,
        );
        check_combat_end(&self.token_balances, combat);
        if combat.is_finished {
            self.last_combat_result = Some(combat.outcome.clone());
            self.combat_results.push(combat.outcome.clone());
            self.current_combat = None;
            self.encounter_phase = super::types::EncounterPhase::Scouting;
        }
        self.draw_player_cards_by_type(atk_draws, def_draws, res_draws, rng);
        Ok(())
    }

    /// Draw player cards from deck to hand per card type, recycling discard if needed.
    fn draw_player_cards_by_type(
        &mut self,
        attack: u32,
        defence: u32,
        resource: u32,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) {
        self.draw_player_cards_of_kind(attack, |k| matches!(k, CardKind::Attack { .. }), rng);
        self.draw_player_cards_of_kind(defence, |k| matches!(k, CardKind::Defence { .. }), rng);
        self.draw_player_cards_of_kind(resource, |k| matches!(k, CardKind::Resource { .. }), rng);
    }

    /// Draw `count` player cards of a specific kind from deck to hand.
    /// Recycles discard→deck for cards matching `kind_filter` when deck is empty.
    fn draw_player_cards_of_kind(
        &mut self,
        count: u32,
        kind_filter: fn(&CardKind) -> bool,
        rng: &mut rand_pcg::Lcg64Xsh32,
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
                let _ = self.library.draw(drawable[pick]);
            } else {
                let pick = (rng.next_u64() as usize) % drawable.len();
                let _ = self.library.draw(drawable[pick]);
            }
        }
    }

    /// Resolve an enemy card play from hand in the current combat phase.
    /// Played cards move to discard. DrawCards effects trigger per-type enemy draws.
    pub fn resolve_enemy_play(&mut self, rng: &mut rand_pcg::Lcg64Xsh32) -> Result<(), String> {
        let combat = self.current_combat.as_ref().ok_or("No active combat")?;
        let phase = combat.phase.clone();

        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
        let deck = match phase {
            super::types::CombatPhase::Attacking => &mut combat.enemy_attack_deck,
            super::types::CombatPhase::Defending => &mut combat.enemy_defence_deck,
            super::types::CombatPhase::Resourcing => &mut combat.enemy_resource_deck,
        };

        // Collect indices of cards with hand > 0
        let hand_indices: Vec<usize> = deck
            .iter()
            .enumerate()
            .filter(|(_, c)| c.counts.hand > 0)
            .map(|(i, _)| i)
            .collect();

        if !hand_indices.is_empty() {
            use rand::RngCore;
            let pick_idx = (rng.next_u64() as usize) % hand_indices.len();
            let card_idx = hand_indices[pick_idx];
            deck[card_idx].counts.hand -= 1;
            deck[card_idx].counts.discard += 1;
            let effect_ids = deck[card_idx].effect_ids.clone();

            let (mut atk_draws, mut def_draws, mut res_draws) = (0u32, 0u32, 0u32);
            for &id in &effect_ids {
                if let Some(super::types::CardEffectKind::DrawCards {
                    attack,
                    defence,
                    resource,
                }) = self.library.resolve_effect(id)
                {
                    atk_draws += attack;
                    def_draws += defence;
                    res_draws += resource;
                }
            }

            apply_card_effects(
                &effect_ids,
                false,
                &mut self.token_balances,
                combat,
                &self.library,
            );
            check_combat_end(&self.token_balances, combat);

            // Handle enemy draws per deck type
            if !combat.is_finished {
                Self::enemy_draw_n(rng, &mut combat.enemy_attack_deck, atk_draws);
                Self::enemy_draw_n(rng, &mut combat.enemy_defence_deck, def_draws);
                Self::enemy_draw_n(rng, &mut combat.enemy_resource_deck, res_draws);
            }

            if combat.is_finished {
                self.last_combat_result = Some(combat.outcome.clone());
                self.combat_results.push(combat.outcome.clone());
                self.current_combat = None;
                self.encounter_phase = super::types::EncounterPhase::Scouting;
            }
        }
        Ok(())
    }

    /// Draw `count` random cards from a single enemy deck to hand, recycling discard if needed.
    fn enemy_draw_n(
        rng: &mut rand_pcg::Lcg64Xsh32,
        deck: &mut [super::types::EnemyCardDef],
        count: u32,
    ) {
        for _ in 0..count {
            Self::enemy_draw_random(rng, deck);
        }
    }

    /// Shuffle enemy hand: move all cards to deck, then draw random cards back to hand.
    fn enemy_shuffle_hand(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::EnemyCardDef]) {
        let target_hand: u32 = deck.iter().map(|c| c.counts.hand).sum();
        // Move all hand cards to deck
        for card in deck.iter_mut() {
            card.counts.deck += card.counts.hand;
            card.counts.hand = 0;
        }
        // Draw random cards until hand is full again
        for _ in 0..target_hand {
            Self::enemy_draw_random(rng, deck);
        }
    }

    /// Draw one random card from enemy deck to hand, recycling discard if needed.
    fn enemy_draw_random(rng: &mut rand_pcg::Lcg64Xsh32, deck: &mut [super::types::EnemyCardDef]) {
        use rand::RngCore;
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
        if total_deck == 0 {
            // Recycle discard to deck
            let total_discard: u32 = deck.iter().map(|c| c.counts.discard).sum();
            if total_discard == 0 {
                return;
            }
            for card in deck.iter_mut() {
                card.counts.deck += card.counts.discard;
                card.counts.discard = 0;
            }
        }
        // Pick a random card from deck (weighted by deck count)
        let total_deck: u32 = deck.iter().map(|c| c.counts.deck).sum();
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

    /// Advance combat phase to next (Defending → Attacking → Resourcing → Defending).
    pub fn advance_combat_phase(&mut self) -> Result<(), String> {
        let combat = self.current_combat.as_mut().ok_or("No active combat")?;
        combat.phase = combat.phase.next();
        Ok(())
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
                    gs.current_combat = None;
                    gs.encounter_phase = new_gs.encounter_phase;
                    gs.last_combat_result = None;
                    gs.combat_results.clear();
                }
                ActionPayload::DrawEncounter { encounter_id, .. } => {
                    if let Ok(card_id) = encounter_id.parse::<usize>() {
                        let health_key =
                            super::types::Token::persistent(super::types::TokenType::Health);
                        if gs.token_balances.get(&health_key).copied().unwrap_or(0) == 0 {
                            gs.token_balances.insert(health_key, 20);
                        }
                        let _ = gs.library.play(card_id);
                        let _ = gs.start_combat(card_id, &mut rng);
                    }
                }
                ActionPayload::PlayCard { card_id, .. } => {
                    let _ = gs.library.play(*card_id);
                    let _ = gs.resolve_player_card(*card_id, &mut rng);
                    if gs.current_combat.is_some() {
                        let _ = gs.resolve_enemy_play(&mut rng);
                        if gs.current_combat.is_some() {
                            let _ = gs.advance_combat_phase();
                        }
                    }
                }
                ActionPayload::ApplyScouting { .. } => {
                    if let Some(ref combat) = gs.current_combat {
                        if let Some(enc_id) = combat.encounter_card_id {
                            let _ = gs.library.return_to_deck(enc_id);
                        }
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
                ActionPayload::GrantToken {
                    token_id, amount, ..
                } => {
                    let v = gs
                        .token_balances
                        .entry(super::types::Token::persistent(token_id.clone()))
                        .or_insert(0);
                    *v += *amount;
                }
                ActionPayload::ConsumeToken {
                    token_id, amount, ..
                } => {
                    let v = gs
                        .token_balances
                        .entry(super::types::Token::persistent(token_id.clone()))
                        .or_insert(0);
                    *v -= *amount;
                }
                ActionPayload::ExpireToken {
                    token_id, amount, ..
                } => {
                    let v = gs
                        .token_balances
                        .entry(super::types::Token::persistent(token_id.clone()))
                        .or_insert(0);
                    *v = (*v - *amount).max(0);
                }
                _ => {
                    // RngDraw, RngSnapshot, etc. are audit entries
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
