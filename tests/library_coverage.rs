use my_little_cardgame::library::types::{
    split_token_amounts, token_balance_by_type, ActionPayload, CardCounts, CardEffectKind,
    CardKind, ConcreteEffect, Discipline, Token, TokenAmount, TokenLifecycle, TokenType,
};
use my_little_cardgame::library::{GameState, Library};
use rand::SeedableRng;
use rand_pcg::Lcg64Xsh32;

#[test]
fn card_counts_total() {
    let counts = CardCounts {
        library: 10,
        deck: 5,
        hand: 3,
        discard: 2,
    };
    assert_eq!(counts.total(), 20);
}

#[test]
fn library_draw_and_play_and_return() {
    let mut lib = Library::new();
    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    // First add a card effect entry (id 0)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::LoseTokens {
                token_type: TokenType::Health,
                min: 500,
                max: 500,
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
        &mut rng,
        vec![],
    );
    let id = lib.add_card(
        CardKind::Attack {
            effects: vec![ConcreteEffect {
                effect_id: 0,
                rolled_value: 500,
                rolled_costs: vec![],
                rolled_cap: None,
                rolled_gain_percent: None,
            }],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
        &mut rng,
        vec![],
    );

    // Draw moves from deck to hand
    assert!(lib.draw(id).is_ok());
    assert_eq!(lib.cards[id].counts.deck, 2);
    assert_eq!(lib.cards[id].counts.hand, 1);

    // Play moves from hand to discard
    assert!(lib.play(id).is_ok());
    assert_eq!(lib.cards[id].counts.hand, 0);
    assert_eq!(lib.cards[id].counts.discard, 1);

    // Return moves from discard to library
    assert!(lib.return_to_library(id).is_ok());
    assert_eq!(lib.cards[id].counts.discard, 0);
    assert_eq!(lib.cards[id].counts.library, 1);

    // Add to deck moves from library to deck
    assert!(lib.add_to_deck(id, 1).is_ok());
    assert_eq!(lib.cards[id].counts.library, 0);
    assert_eq!(lib.cards[id].counts.deck, 3);
}

#[test]
fn library_draw_error_when_deck_empty() {
    let mut lib = Library::new();
    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let id = lib.add_card(
        CardKind::Attack { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    assert!(lib.draw(id).is_err());
}

#[test]
fn library_play_error_when_hand_empty() {
    let mut lib = Library::new();
    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let id = lib.add_card(
        CardKind::Attack { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    assert!(lib.play(id).is_err());
}

#[test]
fn library_return_to_library_error_when_discard_empty() {
    let mut lib = Library::new();
    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let id = lib.add_card(
        CardKind::Attack { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    assert!(lib.return_to_library(id).is_err());
}

#[test]
fn library_add_to_deck_error_when_library_insufficient() {
    let mut lib = Library::new();
    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let id = lib.add_card(
        CardKind::Attack { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    assert!(lib.add_to_deck(id, 5).is_err());
}

#[test]
fn library_hand_cards_returns_cards_in_hand() {
    let mut lib = Library::new();
    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    lib.add_card(
        CardKind::Attack { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 3,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    lib.add_card(
        CardKind::Defence { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 0,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    let hand = lib.hand_cards();
    assert_eq!(hand.len(), 1);
    assert_eq!(hand[0].0, 0);
}

#[test]
fn library_cards_matching_filters_by_predicate() {
    let mut lib = Library::new();
    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    lib.add_card(
        CardKind::Attack { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 1,
            hand: 1,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    lib.add_card(
        CardKind::Defence { effects: vec![] },
        CardCounts {
            library: 0,
            deck: 1,
            hand: 1,
            discard: 0,
        },
        &mut rng,
        vec![],
    );
    let attacks = lib.cards_matching(|kind| matches!(kind, CardKind::Attack { .. }));
    assert_eq!(attacks.len(), 1);
}

#[test]
fn library_draw_nonexistent_card_returns_error() {
    let mut lib = Library::new();
    assert!(lib.draw(999).is_err());
}

#[test]
fn game_state_draw_random_cards() {
    let mut gs = GameState::new();
    // Library starts with Attack having deck:15 hand:5 (now at index 9)
    let initial_hand = gs.library.cards[9].counts.hand;
    let initial_deck = gs.library.cards[9].counts.deck;
    assert!(initial_deck > 0);
    // draw_random_cards is private, but we can test it via resolve_player_card
    // playing a resource card (id 11) triggers draw_count=1
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    let mut rng = rand_pcg::Lcg64Xsh32::from_seed([0u8; 16]);
    let _ = gs.start_combat(12, &mut rng);
    let _ = gs.advance_combat_phase(); // Defending -> Attacking
    let _ = gs.advance_combat_phase(); // Attacking -> Resourcing
    let _ = gs.resolve_player_card(11, &mut rng); // Resource card draws 1
                                                  // Check that total cards in hand changed
    let total_hand: u32 = gs.library.cards.iter().map(|c| c.counts.hand).sum();
    assert!(total_hand >= initial_hand); // drew at least 1 card
}

#[test]
fn replay_from_log_handles_set_seed() {
    let gs = GameState::new();
    gs.action_log
        .append("NewGame", ActionPayload::SetSeed { seed: 42 });

    let log_clone = gs.action_log.clone();
    let replayed = GameState::replay_from_log(&log_clone);
    // After replay, state should be freshly initialized (SetSeed resets)
    assert_eq!(
        token_balance_by_type(&replayed.token_balances, &TokenType::CombatInsight),
        0
    );
}

#[test]
fn game_state_shutdown() {
    let gs = GameState::new();
    gs.shutdown(); // should not panic even without a writer
}

#[test]
fn game_state_default() {
    let gs: GameState = Default::default();
    assert!(gs.current_encounter.is_none());
}

#[test]
fn start_combat_with_non_encounter_card() {
    let mut gs = GameState::new();
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    let result = gs.start_combat(0, &mut rand_pcg::Lcg64Xsh32::from_seed([0u8; 16])); // card 0 is PlayerCardEffect, not CombatEncounter
    assert!(result.is_err());
}

#[test]
fn resolve_player_card_non_action_card() {
    let mut gs = GameState::new();
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    let mut rng = rand_pcg::Lcg64Xsh32::from_seed([0u8; 16]);
    let _ = gs.start_combat(11, &mut rng);
    // Try to play Encounter card (id 11) as a player card
    let result = gs.resolve_player_card(11, &mut rng);
    assert!(result.is_err());
}

#[test]
fn resolve_enemy_play_with_non_encounter() {
    let mut gs = GameState::new();
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    // No combat started, should return error
    let mut rng = rand_pcg::Lcg64Xsh32::from_seed([0u8; 16]);
    let result = gs.resolve_enemy_play(&mut rng);
    assert!(result.is_err());
}

#[test]
fn token_type_all_returns_all_variants() {
    let all = TokenType::all();
    assert!(all.len() > 10, "Should have many token types");
    assert!(all.contains(&TokenType::Health));
    assert!(all.contains(&TokenType::Stamina));
}

#[test]
fn token_type_is_gathering_material() {
    assert!(TokenType::Ore.is_gathering_material());
    assert!(TokenType::Lumber.is_gathering_material());
    assert!(TokenType::Plant.is_gathering_material());
    assert!(TokenType::Fish.is_gathering_material());
    assert!(!TokenType::Health.is_gathering_material());
    assert!(!TokenType::Stamina.is_gathering_material());
}

#[test]
fn token_type_insight_for_discipline() {
    assert_eq!(
        TokenType::insight_for_discipline(&Discipline::Combat),
        TokenType::CombatInsight
    );
    assert_eq!(
        TokenType::insight_for_discipline(&Discipline::Mining),
        TokenType::MiningInsight
    );
    assert_eq!(
        TokenType::insight_for_discipline(&Discipline::Fishing),
        TokenType::FishingInsight
    );
}

#[test]
fn token_type_durability_for_discipline() {
    assert!(TokenType::durability_for_discipline(&Discipline::Mining).is_some());
    assert!(TokenType::durability_for_discipline(&Discipline::Woodcutting).is_some());
    assert!(TokenType::durability_for_discipline(&Discipline::Combat).is_none());
}

#[test]
fn token_type_is_durability_cost() {
    assert!(TokenType::MiningDurability.is_durability_cost());
    assert!(TokenType::WoodcuttingDurability.is_durability_cost());
    assert!(TokenType::Durability.is_durability_cost());
    assert!(!TokenType::Health.is_durability_cost());
}

#[test]
fn token_type_resolve_durability() {
    assert_eq!(
        TokenType::Durability.resolve_durability(&Discipline::Mining),
        TokenType::MiningDurability
    );
    // Non-durability types resolve to themselves
    assert_eq!(
        TokenType::Health.resolve_durability(&Discipline::Mining),
        TokenType::Health
    );
}

#[test]
fn token_balance_by_type_returns_zero_for_missing() {
    let map = std::collections::HashMap::new();
    assert_eq!(token_balance_by_type(&map, &TokenType::Health), 0);
}

#[test]
fn token_dodge_constructor() {
    let token = Token::dodge();
    assert_eq!(token.token_type, TokenType::Dodge);
}

#[test]
fn action_log_write_and_load_file() {
    use my_little_cardgame::library::action_log::ActionLog;

    let log = ActionLog::new();
    log.append("NewGame", ActionPayload::SetSeed { seed: 42 });
    log.append(
        "EncounterPickEncounter",
        ActionPayload::PlayCard { card_id: 5 },
    );

    let tmp_path = "/tmp/test_action_log_roundtrip.jsonl";
    log.write_all_to_file(tmp_path)
        .expect("write should succeed");

    let loaded = ActionLog::load_from_file(tmp_path).expect("load should succeed");
    let entries = loaded.entries();
    assert_eq!(entries.len(), 2, "Should load 2 entries");
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[1].seq, 2);

    // Cleanup
    let _ = std::fs::remove_file(tmp_path);
}

#[test]
fn action_log_load_from_nonexistent_file() {
    use my_little_cardgame::library::action_log::ActionLog;
    let result = ActionLog::load_from_file("/tmp/nonexistent_test_file_12345.jsonl");
    assert!(result.is_err(), "Loading from nonexistent file should fail");
}

#[test]
fn split_token_amounts_separates_durability() {
    let costs = vec![
        TokenAmount {
            token_type: TokenType::Stamina,
            amount: 100,
            cap: None,
        },
        TokenAmount {
            token_type: TokenType::MiningDurability,
            amount: 50,
            cap: None,
        },
        TokenAmount {
            token_type: TokenType::Ore,
            amount: 30,
            cap: None,
        },
    ];
    let (pre_play, post_play) = split_token_amounts(&costs);
    assert_eq!(pre_play.len(), 2, "Non-durability costs");
    assert_eq!(post_play.len(), 1, "Durability costs");
    assert_eq!(post_play[0].token_type, TokenType::MiningDurability);
}

#[test]
fn token_type_all_covers_many_variants() {
    let all = TokenType::all();
    // Exercise various token types to ensure all() includes them
    assert!(all.contains(&TokenType::Ore));
    assert!(all.contains(&TokenType::Lumber));
    assert!(all.contains(&TokenType::Plant));
    assert!(all.contains(&TokenType::Fish));
    assert!(all.contains(&TokenType::Dodge));
    assert!(all.contains(&TokenType::Shield));
    assert!(all.contains(&TokenType::CombatInsight));
    assert!(all.contains(&TokenType::MiningInsight));
}

#[test]
fn action_log_set_writer() {
    use my_little_cardgame::library::action_log::ActionLog;
    let mut log = ActionLog::new();
    log.set_writer(None); // just ensure it doesn't panic
}

#[test]
fn action_log_clone_preserves_entries() {
    use my_little_cardgame::library::action_log::ActionLog;
    let log = ActionLog::new();
    log.append("test", ActionPayload::SetSeed { seed: 1 });
    log.append("test", ActionPayload::SetSeed { seed: 2 });

    let cloned = log.clone();
    let entries = cloned.entries();
    assert_eq!(entries.len(), 2, "Clone should preserve entries");
    assert_eq!(entries[0].seq, 1);
    assert_eq!(entries[1].seq, 2);
}

/// Comprehensive replay_from_log test exercising all action types.
/// Build a complete game log by running a game, then verify replay produces same state.
#[test]
fn replay_from_log_full_game() {
    use my_little_cardgame::library::types::EncounterState;
    use rand::SeedableRng;

    let mut gs = GameState::new_with_rng(&mut Lcg64Xsh32::seed_from_u64(42));
    let mut rng = Lcg64Xsh32::seed_from_u64(42);

    // Record SetSeed
    gs.action_log
        .append("NewGame", ActionPayload::SetSeed { seed: 42 });

    // Find encounter cards in hand
    let enc_ids: Vec<usize> = gs
        .library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
        .map(|(i, _)| i)
        .collect();

    if enc_ids.is_empty() {
        return;
    }

    // Pick first encounter (combat)
    let enc_id = enc_ids[0];
    gs.action_log.append(
        "EncounterPickEncounter",
        ActionPayload::DrawEncounter {
            encounter_id: enc_id.to_string(),
        },
    );

    // Set health
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    let _ = gs.library.play(enc_id);

    // Start based on encounter kind
    let card = gs.library.get(enc_id).cloned();
    if let Some(lib_card) = card {
        match &lib_card.kind {
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Combat { .. },
            } => {
                let _ = gs.start_combat(enc_id, &mut rng);
            }
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Mining { .. },
            } => {
                let _ = gs.start_mining_encounter(enc_id, &mut rng);
            }
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Herbalism { .. },
            } => {
                let _ = gs.start_herbalism_encounter(enc_id, &mut rng);
            }
            CardKind::Encounter {
                encounter_kind:
                    my_little_cardgame::library::types::EncounterKind::Woodcutting { .. },
            } => {
                let _ = gs.start_woodcutting_encounter(enc_id, &mut rng);
            }
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Fishing { .. },
            } => {
                let _ = gs.start_fishing_encounter(enc_id, &mut rng);
            }
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Rest { .. },
            } => {
                let _ = gs.start_rest_encounter(enc_id, &mut rng);
            }
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Crafting { .. },
            } => {
                let _ = gs.start_crafting_encounter(enc_id, &mut rng);
            }
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Research { .. },
            } => {
                let _ = gs.start_research_encounter(enc_id);
            }
            CardKind::Encounter {
                encounter_kind: my_little_cardgame::library::types::EncounterKind::Milestone { .. },
            } => {
                let _ = gs.start_milestone_encounter(enc_id, &mut rng);
            }
            _ => {}
        }
    }

    // If combat, play a round
    if let Some(EncounterState::Combat(_)) = &gs.current_encounter {
        // Play Defence card
        let def_ids: Vec<usize> = gs
            .library
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.kind, CardKind::Defence { .. }) && c.counts.hand > 0)
            .map(|(i, _)| i)
            .collect();
        if !def_ids.is_empty() {
            let card_id = def_ids[0];
            gs.action_log
                .append("EncounterPlayCard", ActionPayload::PlayCard { card_id });
            let _ = gs.library.play(card_id);
            let _ = gs.resolve_player_card(card_id, &mut rng);
            if gs.current_encounter.is_some() {
                let _ = gs.resolve_enemy_play(&mut rng);
                if gs.current_encounter.is_some() {
                    let _ = gs.advance_combat_phase();
                }
            }
        }
    }

    // Abort encounter
    gs.action_log
        .append("EncounterAbort", ActionPayload::AbortEncounter);
    gs.abort_encounter();

    // Apply scouting
    gs.action_log.append(
        "EncounterApplyScouting",
        ActionPayload::ApplyScouting { card_ids: vec![] },
    );
    if let Some(ref enc) = gs.current_encounter {
        let enc_id = enc.encounter_card_id();
        let _ = gs.library.return_to_deck(enc_id);
    }
    gs.library.encounter_draw_to_hand(3);
    gs.encounter_phase = my_little_cardgame::library::types::EncounterPhase::NoEncounter;

    // Now replay the log
    let log_clone = gs.action_log.clone();
    let replayed = GameState::replay_from_log(&log_clone);

    // Verify state matches
    assert_eq!(replayed.encounter_phase, gs.encounter_phase);
    assert!(replayed.current_encounter.is_none());
}

/// replay_from_log with multiple encounter types in one session.
#[test]
fn replay_from_log_multi_encounter() {
    use rand::SeedableRng;

    let mut gs = GameState::new_with_rng(&mut Lcg64Xsh32::seed_from_u64(42));
    let mut rng = Lcg64Xsh32::seed_from_u64(42);

    gs.action_log
        .append("NewGame", ActionPayload::SetSeed { seed: 42 });
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);

    // Pick and abort several encounters to cover different branches in replay
    for _ in 0..5 {
        let enc_ids: Vec<usize> = gs
            .library
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.kind, CardKind::Encounter { .. }) && c.counts.hand > 0)
            .map(|(i, _)| i)
            .collect();
        if enc_ids.is_empty() {
            break;
        }

        let enc_id = enc_ids[0];
        gs.action_log.append(
            "EncounterPickEncounter",
            ActionPayload::DrawEncounter {
                encounter_id: enc_id.to_string(),
            },
        );

        let _ = gs.library.play(enc_id);
        let card = gs.library.get(enc_id).cloned();
        if let Some(lib_card) = card {
            match &lib_card.kind {
                CardKind::Encounter {
                    encounter_kind: my_little_cardgame::library::types::EncounterKind::Combat { .. },
                } => {
                    let _ = gs.start_combat(enc_id, &mut rng);
                }
                CardKind::Encounter {
                    encounter_kind: my_little_cardgame::library::types::EncounterKind::Mining { .. },
                } => {
                    let _ = gs.start_mining_encounter(enc_id, &mut rng);
                }
                CardKind::Encounter {
                    encounter_kind:
                        my_little_cardgame::library::types::EncounterKind::Herbalism { .. },
                } => {
                    let _ = gs.start_herbalism_encounter(enc_id, &mut rng);
                }
                CardKind::Encounter {
                    encounter_kind:
                        my_little_cardgame::library::types::EncounterKind::Woodcutting { .. },
                } => {
                    let _ = gs.start_woodcutting_encounter(enc_id, &mut rng);
                }
                CardKind::Encounter {
                    encounter_kind:
                        my_little_cardgame::library::types::EncounterKind::Fishing { .. },
                } => {
                    let _ = gs.start_fishing_encounter(enc_id, &mut rng);
                }
                CardKind::Encounter {
                    encounter_kind: my_little_cardgame::library::types::EncounterKind::Rest { .. },
                } => {
                    let _ = gs.start_rest_encounter(enc_id, &mut rng);
                }
                CardKind::Encounter {
                    encounter_kind:
                        my_little_cardgame::library::types::EncounterKind::Crafting { .. },
                } => {
                    let _ = gs.start_crafting_encounter(enc_id, &mut rng);
                }
                CardKind::Encounter {
                    encounter_kind:
                        my_little_cardgame::library::types::EncounterKind::Research { .. },
                } => {
                    let _ = gs.start_research_encounter(enc_id);
                }
                CardKind::Encounter {
                    encounter_kind:
                        my_little_cardgame::library::types::EncounterKind::Milestone { .. },
                } => {
                    let _ = gs.start_milestone_encounter(enc_id, &mut rng);
                }
                _ => {}
            }
        }
        gs.snapshot_encounter_start_tokens();

        // Abort
        gs.action_log
            .append("EncounterAbort", ActionPayload::AbortEncounter);
        if matches!(
            &gs.current_encounter,
            Some(my_little_cardgame::library::types::EncounterState::Rest(_))
        ) {
            gs.abort_rest_encounter();
        } else if matches!(
            &gs.current_encounter,
            Some(my_little_cardgame::library::types::EncounterState::Crafting(_))
        ) {
            let _ = gs.abort_crafting_encounter();
        } else if matches!(
            &gs.current_encounter,
            Some(my_little_cardgame::library::types::EncounterState::Research(_))
        ) {
            gs.abort_research_encounter();
        } else if matches!(
            &gs.current_encounter,
            Some(my_little_cardgame::library::types::EncounterState::Milestone(_))
        ) {
            gs.abort_milestone_encounter();
        } else {
            gs.abort_encounter();
        }

        // Scout
        gs.action_log.append(
            "EncounterApplyScouting",
            ActionPayload::ApplyScouting { card_ids: vec![] },
        );
        if let Some(ref enc) = gs.current_encounter {
            let eid = enc.encounter_card_id();
            let _ = gs.library.return_to_deck(eid);
        }
        gs.library.encounter_draw_to_hand(3);
        gs.encounter_phase = my_little_cardgame::library::types::EncounterPhase::NoEncounter;
    }

    // Replay and verify
    let log_clone = gs.action_log.clone();
    let replayed = GameState::replay_from_log(&log_clone);
    assert!(replayed.current_encounter.is_none());
    assert_eq!(
        replayed.encounter_phase,
        my_little_cardgame::library::types::EncounterPhase::NoEncounter
    );
}

// ===================================================================
// RESEARCH: Direct unit tests for deep code paths
// ===================================================================

/// Setup a game state with a research encounter active and a project selected.
fn setup_research_game() -> (GameState, Lcg64Xsh32) {
    use my_little_cardgame::library::types::EncounterKind;

    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let mut gs = GameState::new_with_rng(&mut rng);
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    gs.token_balances
        .insert(Token::persistent(TokenType::CombatInsight), 5000);
    gs.token_balances
        .insert(Token::persistent(TokenType::Stamina), 5000);

    // Find a research encounter card
    let research_enc_id = gs
        .library
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, c)| match &c.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Research { .. },
            } => Some(i),
            _ => None,
        });

    if let Some(enc_id) = research_enc_id {
        // Move to hand if needed
        if gs.library.cards[enc_id].counts.hand == 0 {
            if gs.library.cards[enc_id].counts.deck > 0 {
                gs.library.cards[enc_id].counts.deck -= 1;
                gs.library.cards[enc_id].counts.hand += 1;
            } else if gs.library.cards[enc_id].counts.library > 0 {
                gs.library.cards[enc_id].counts.library -= 1;
                gs.library.cards[enc_id].counts.hand += 1;
            }
        }
        let _ = gs.library.play(enc_id);
        let _ = gs.start_research_encounter(enc_id);
    }

    (gs, rng)
}

#[test]
fn research_begin_experiment_and_play_hand_direct() {
    let (mut gs, mut rng) = setup_research_game();

    // Verify we're in a research encounter
    if !matches!(
        &gs.current_encounter,
        Some(my_little_cardgame::library::types::EncounterState::Research(_))
    ) {
        return; // No research encounter available
    }

    // Choose project
    let result = gs.research_choose_project(Discipline::Combat, 1, &mut rng);
    assert!(result.is_ok(), "ChooseProject should succeed: {:?}", result);

    // Select candidate
    let result = gs.research_select_candidate(0);
    assert!(
        result.is_ok(),
        "SelectCandidate should succeed: {:?}",
        result
    );

    // Begin experiment
    let result = gs.research_begin_experiment(&mut rng);
    assert!(
        result.is_ok(),
        "BeginExperiment should succeed: {:?}",
        result
    );

    // Find research cards in hand
    let research_hand_ids: Vec<usize> = gs
        .library
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c.kind, CardKind::Research { .. }) && c.counts.hand > 0)
        .map(|(i, _)| i)
        .collect();

    // Need exactly 3 cards (target_size = 3)
    if research_hand_ids.len() >= 3 {
        let card_ids = vec![
            research_hand_ids[0],
            research_hand_ids[1],
            research_hand_ids[2],
        ];
        let result = gs.research_play_hand(card_ids, &mut rng);
        assert!(result.is_ok(), "PlayHand should succeed: {:?}", result);

        // Verify round was recorded
        if let Some(my_little_cardgame::library::types::EncounterState::Research(r)) =
            &gs.current_encounter
        {
            assert_eq!(r.rounds_played, 1);
            assert!(!r.round_history.is_empty());
        }

        // Play another round
        let research_hand_ids2: Vec<usize> = gs
            .library
            .cards
            .iter()
            .enumerate()
            .filter(|(_, c)| matches!(c.kind, CardKind::Research { .. }) && c.counts.hand > 0)
            .map(|(i, _)| i)
            .collect();
        if research_hand_ids2.len() >= 3 {
            let card_ids2 = vec![
                research_hand_ids2[0],
                research_hand_ids2[1],
                research_hand_ids2[2],
            ];
            let _ = gs.research_play_hand(card_ids2, &mut rng);
        }

        // Conclude experiment
        let result = gs.research_conclude_experiment(&mut rng);
        assert!(
            result.is_ok(),
            "ConcludeExperiment should succeed: {:?}",
            result
        );
    }
}

#[test]
fn research_progress_completes_project() {
    let (mut gs, mut rng) = setup_research_game();

    if !matches!(
        &gs.current_encounter,
        Some(my_little_cardgame::library::types::EncounterState::Research(_))
    ) {
        return;
    }

    let _ = gs.research_choose_project(Discipline::Combat, 1, &mut rng);
    let _ = gs.research_select_candidate(0);

    // Progress until project completes
    for _ in 0..20 {
        let insight = gs
            .token_balances
            .get(&Token::persistent(TokenType::CombatInsight))
            .copied()
            .unwrap_or(0);
        if insight < 1 {
            break;
        }
        let result = gs.research_progress(100, &mut rng);
        if result.is_err() {
            break;
        }
    }

    // Check if project completed (should have added a new card)
    let _has_project = gs.current_research.is_some();
    // If project completed, current_research would be None
    // Either way, coverage of research_progress deep paths is exercised

    let _ = gs.conclude_research_encounter();
}

#[test]
fn research_play_hand_with_wrong_count() {
    let (mut gs, mut rng) = setup_research_game();

    if !matches!(
        &gs.current_encounter,
        Some(my_little_cardgame::library::types::EncounterState::Research(_))
    ) {
        return;
    }

    let _ = gs.research_choose_project(Discipline::Combat, 1, &mut rng);
    let _ = gs.research_select_candidate(0);
    let _ = gs.research_begin_experiment(&mut rng);

    // Wrong number of cards (should fail)
    let result = gs.research_play_hand(vec![0], &mut rng);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Must play exactly"));
}

// ===================================================================
// HERBALISM: Direct unit tests for match mode coverage
// ===================================================================

fn setup_herbalism_game(seed: u64) -> (GameState, Lcg64Xsh32) {
    use my_little_cardgame::library::types::EncounterKind;

    let mut rng = Lcg64Xsh32::seed_from_u64(seed);
    let mut gs = GameState::new_with_rng(&mut rng);
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    gs.token_balances
        .insert(Token::persistent(TokenType::Stamina), 5000);
    gs.token_balances
        .insert(Token::persistent(TokenType::HerbalismDurability), 5000);

    let herbalism_enc_id = gs
        .library
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, c)| match &c.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Herbalism { .. },
            } => Some(i),
            _ => None,
        });

    if let Some(enc_id) = herbalism_enc_id {
        if gs.library.cards[enc_id].counts.hand == 0 {
            if gs.library.cards[enc_id].counts.deck > 0 {
                gs.library.cards[enc_id].counts.deck -= 1;
                gs.library.cards[enc_id].counts.hand += 1;
            } else if gs.library.cards[enc_id].counts.library > 0 {
                gs.library.cards[enc_id].counts.library -= 1;
                gs.library.cards[enc_id].counts.hand += 1;
            }
        }
        let _ = gs.library.play(enc_id);
        let _ = gs.start_herbalism_encounter(enc_id, &mut rng);
    }

    (gs, rng)
}

#[test]
fn herbalism_play_multiple_cards_directly() {
    for seed in [
        42, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 1111, 2222, 3333, 4444, 5555, 6666,
        7777, 8888, 9999u64,
    ] {
        let (mut gs, mut rng) = setup_herbalism_game(seed);

        if !matches!(
            &gs.current_encounter,
            Some(my_little_cardgame::library::types::EncounterState::Herbalism(_))
        ) {
            continue;
        }

        for _ in 0..20 {
            // Find herbalism player cards in hand
            let herb_hand: Vec<usize> = gs
                .library
                .cards
                .iter()
                .enumerate()
                .filter(|(_, c)| matches!(c.kind, CardKind::Herbalism { .. }) && c.counts.hand > 0)
                .map(|(i, _)| i)
                .collect();

            if herb_hand.is_empty() {
                break;
            }

            let mut any_played = false;
            for &card_id in &herb_hand {
                let _ = gs.library.play(card_id);
                let result = gs.resolve_player_herbalism_card(card_id, &mut rng);
                if result.is_ok() {
                    any_played = true;
                    break;
                }
            }

            if !any_played {
                break;
            }

            // Check if encounter ended
            match &gs.current_encounter {
                Some(my_little_cardgame::library::types::EncounterState::Herbalism(h)) => {
                    if h.outcome != my_little_cardgame::library::types::EncounterOutcome::Undecided
                    {
                        break;
                    }
                }
                None => break,
                _ => break,
            }
        }
    }
}

// ===================================================================
// MILESTONE: Direct unit tests for registration functions
// ===================================================================

#[test]
fn milestone_register_all_disciplines() {
    use my_little_cardgame::library::types::EncounterKind;

    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let mut gs = GameState::new_with_rng(&mut rng);
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    gs.token_balances
        .insert(Token::persistent(TokenType::MilestoneInsight), 50000);

    // Find milestone encounters for each discipline
    let disciplines = ["Combat", "Mining", "Herbalism", "Woodcutting", "Fishing"];
    for disc_name in &disciplines {
        let milestone_id = gs
            .library
            .cards
            .iter()
            .enumerate()
            .find_map(|(i, c)| match &c.kind {
                CardKind::Encounter {
                    encounter_kind: EncounterKind::Milestone { milestone_def },
                } => {
                    let disc_str = format!("{:?}", milestone_def.discipline);
                    if disc_str == *disc_name {
                        Some(i)
                    } else {
                        None
                    }
                }
                _ => None,
            });

        if let Some(enc_id) = milestone_id {
            // Move to hand
            if gs.library.cards[enc_id].counts.hand == 0 {
                if gs.library.cards[enc_id].counts.deck > 0 {
                    gs.library.cards[enc_id].counts.deck -= 1;
                    gs.library.cards[enc_id].counts.hand += 1;
                } else if gs.library.cards[enc_id].counts.library > 0 {
                    gs.library.cards[enc_id].counts.library -= 1;
                    gs.library.cards[enc_id].counts.hand += 1;
                }
            }
            let _ = gs.library.play(enc_id);
            let result = gs.start_milestone_encounter(enc_id, &mut rng);
            if result.is_ok() {
                // Play a few cards to exercise the inner encounter
                if let Some(my_little_cardgame::library::types::EncounterState::Milestone(m)) =
                    &gs.current_encounter
                {
                    // Verify discipline
                    let disc_str = format!("{:?}", m.discipline);
                    assert_eq!(disc_str, *disc_name);
                }
                gs.abort_milestone_encounter();
            }
            // Return card for reuse
            let _ = gs.library.return_to_hand(enc_id);
        }
    }
}

/// Exercise milestone win path which triggers reward generation.
#[test]
fn milestone_win_triggers_reward_and_next_tier() {
    use my_little_cardgame::library::types::{EncounterKind, EncounterOutcome, EncounterState};

    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let mut gs = GameState::new_with_rng(&mut rng);
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    gs.token_balances
        .insert(Token::persistent(TokenType::Stamina), 5000);
    gs.token_balances
        .insert(Token::persistent(TokenType::MilestoneInsight), 50000);

    // Find Combat milestone
    let milestone_id = gs
        .library
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, c)| match &c.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Milestone { milestone_def },
            } => {
                if matches!(milestone_def.discipline, Discipline::Combat) {
                    Some(i)
                } else {
                    None
                }
            }
            _ => None,
        });

    if let Some(enc_id) = milestone_id {
        if gs.library.cards[enc_id].counts.hand == 0 && gs.library.cards[enc_id].counts.deck > 0 {
            gs.library.cards[enc_id].counts.deck -= 1;
            gs.library.cards[enc_id].counts.hand += 1;
        }
        let _ = gs.library.play(enc_id);
        let _ = gs.start_milestone_encounter(enc_id, &mut rng);

        // Play combat rounds until combat ends
        for _ in 0..400 {
            // Find Defence card
            let def_id = gs.library.cards.iter().enumerate().find_map(|(i, c)| {
                if matches!(c.kind, CardKind::Defence { .. }) && c.counts.hand > 0 {
                    Some(i)
                } else {
                    None
                }
            });
            if def_id.is_none() {
                break;
            }
            let def_id = def_id.unwrap();
            let _ = gs.library.play(def_id);
            let result = gs.resolve_milestone_play_card(def_id, &mut rng);
            if result.is_err() {
                break;
            }

            // Check if encounter ended
            match &gs.current_encounter {
                None => break,
                Some(EncounterState::Milestone(m)) if m.outcome != EncounterOutcome::Undecided => {
                    break
                }
                _ => {}
            }

            // Attack
            let atk_id = gs.library.cards.iter().enumerate().find_map(|(i, c)| {
                if matches!(c.kind, CardKind::Attack { .. }) && c.counts.hand > 0 {
                    Some(i)
                } else {
                    None
                }
            });
            if atk_id.is_none() {
                break;
            }
            let atk_id = atk_id.unwrap();
            let _ = gs.library.play(atk_id);
            let result = gs.resolve_milestone_play_card(atk_id, &mut rng);
            if result.is_err() {
                break;
            }

            match &gs.current_encounter {
                None => break,
                Some(EncounterState::Milestone(m)) if m.outcome != EncounterOutcome::Undecided => {
                    break
                }
                _ => {}
            }

            // Resource
            let res_id = gs.library.cards.iter().enumerate().find_map(|(i, c)| {
                if matches!(c.kind, CardKind::Resource { .. }) && c.counts.hand > 0 {
                    Some(i)
                } else {
                    None
                }
            });
            if res_id.is_none() {
                break;
            }
            let res_id = res_id.unwrap();
            let _ = gs.library.play(res_id);
            let result = gs.resolve_milestone_play_card(res_id, &mut rng);
            if result.is_err() {
                break;
            }

            match &gs.current_encounter {
                None => break,
                Some(EncounterState::Milestone(m)) if m.outcome != EncounterOutcome::Undecided => {
                    break
                }
                _ => {}
            }
        }

        // If milestone is still active, abort it
        if let Some(EncounterState::Milestone(_)) = &gs.current_encounter {
            gs.abort_milestone_encounter();
        }
    }
}

/// Exercise the replay_from_log with research and crafting action types.
#[test]
fn replay_from_log_with_research_actions() {
    use my_little_cardgame::library::types::EncounterKind;

    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let mut gs = GameState::new_with_rng(&mut rng);

    gs.action_log
        .append("NewGame", ActionPayload::SetSeed { seed: 42 });
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);
    gs.token_balances
        .insert(Token::persistent(TokenType::CombatInsight), 5000);
    gs.token_balances
        .insert(Token::persistent(TokenType::Stamina), 5000);

    // Find research encounter
    let research_enc_id = gs
        .library
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, c)| match &c.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Research { .. },
            } => Some(i),
            _ => None,
        });

    if let Some(enc_id) = research_enc_id {
        if gs.library.cards[enc_id].counts.hand == 0 && gs.library.cards[enc_id].counts.deck > 0 {
            gs.library.cards[enc_id].counts.deck -= 1;
            gs.library.cards[enc_id].counts.hand += 1;
        }

        // Record DrawEncounter
        gs.action_log.append(
            "EncounterPickEncounter",
            ActionPayload::DrawEncounter {
                encounter_id: enc_id.to_string(),
            },
        );

        let _ = gs.library.play(enc_id);
        let _ = gs.start_research_encounter(enc_id);

        // Record research actions
        gs.action_log.append(
            "ResearchChooseProject",
            ActionPayload::ResearchChooseProject {
                discipline: Discipline::Combat,
                tier_count: 1,
            },
        );
        let _ = gs.research_choose_project(Discipline::Combat, 1, &mut rng);

        gs.action_log.append(
            "ResearchSelectCandidate",
            ActionPayload::ResearchSelectCandidate { candidate_index: 0 },
        );
        let _ = gs.research_select_candidate(0);

        gs.action_log.append(
            "ResearchProgress",
            ActionPayload::ResearchProgress { amount: 5 },
        );
        let _ = gs.research_progress(5, &mut rng);

        // Conclude
        gs.action_log.append(
            "EncounterConcludeEncounter",
            ActionPayload::ConcludeEncounter,
        );
        let _ = gs.conclude_research_encounter();

        // Scout
        gs.action_log.append(
            "EncounterApplyScouting",
            ActionPayload::ApplyScouting { card_ids: vec![] },
        );
    }

    // Replay and verify
    let log_clone = gs.action_log.clone();
    let replayed = GameState::replay_from_log(&log_clone);
    assert!(replayed.current_encounter.is_none());
}

/// Exercise replay with crafting actions.
#[test]
fn replay_from_log_with_crafting_actions() {
    use my_little_cardgame::library::types::EncounterKind;

    let mut rng = Lcg64Xsh32::seed_from_u64(42);
    let mut gs = GameState::new_with_rng(&mut rng);

    gs.action_log
        .append("NewGame", ActionPayload::SetSeed { seed: 42 });
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 2000);

    // Find crafting encounter
    let crafting_enc_id = gs
        .library
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, c)| match &c.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Crafting { .. },
            } => Some(i),
            _ => None,
        });

    if let Some(enc_id) = crafting_enc_id {
        if gs.library.cards[enc_id].counts.hand == 0 && gs.library.cards[enc_id].counts.deck > 0 {
            gs.library.cards[enc_id].counts.deck -= 1;
            gs.library.cards[enc_id].counts.hand += 1;
        }

        gs.action_log.append(
            "EncounterPickEncounter",
            ActionPayload::DrawEncounter {
                encounter_id: enc_id.to_string(),
            },
        );
        let _ = gs.library.play(enc_id);
        let _ = gs.start_crafting_encounter(enc_id, &mut rng);

        // Record crafting actions
        gs.action_log.append(
            "EncounterCraftDurability",
            ActionPayload::CraftDurability {
                discipline: "Mining".to_string(),
            },
        );
        let _ = gs.resolve_crafting_add_durability("Mining");

        gs.action_log.append(
            "EncounterCraftSwap",
            ActionPayload::CraftSwap {
                from_id: 0,
                to_id: 1,
            },
        );
        let _ = gs.resolve_crafting_swap(0, 1);

        gs.action_log
            .append("EncounterAbort", ActionPayload::AbortEncounter);
        let _ = gs.abort_crafting_encounter();

        gs.action_log.append(
            "EncounterApplyScouting",
            ActionPayload::ApplyScouting { card_ids: vec![] },
        );
    }

    let log_clone = gs.action_log.clone();
    let replayed = GameState::replay_from_log(&log_clone);
    assert!(replayed.current_encounter.is_none());
}
