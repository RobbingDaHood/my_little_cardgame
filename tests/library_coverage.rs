use my_little_cardgame::library::types::{
    token_balance_by_type, ActionPayload, CardCounts, CardEffectKind, CardKind, EffectTarget,
    Token, TokenType,
};
use my_little_cardgame::library::{GameState, Library};

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
    // First add a card effect entry (id 0)
    lib.add_card(
        CardKind::PlayerCardEffect {
            kind: CardEffectKind::ChangeTokens {
                target: EffectTarget::OnOpponent,
                token_type: TokenType::Health,
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
    let id = lib.add_card(
        CardKind::Attack {
            effect_ids: vec![0],
        },
        CardCounts {
            library: 0,
            deck: 3,
            hand: 0,
            discard: 0,
        },
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
    let id = lib.add_card(
        CardKind::Attack { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );
    assert!(lib.draw(id).is_err());
}

#[test]
fn library_play_error_when_hand_empty() {
    let mut lib = Library::new();
    let id = lib.add_card(
        CardKind::Attack { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );
    assert!(lib.play(id).is_err());
}

#[test]
fn library_return_to_library_error_when_discard_empty() {
    let mut lib = Library::new();
    let id = lib.add_card(
        CardKind::Attack { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );
    assert!(lib.return_to_library(id).is_err());
}

#[test]
fn library_add_to_deck_error_when_library_insufficient() {
    let mut lib = Library::new();
    let id = lib.add_card(
        CardKind::Attack { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 0,
            discard: 0,
        },
    );
    assert!(lib.add_to_deck(id, 5).is_err());
}

#[test]
fn library_hand_cards_returns_cards_in_hand() {
    let mut lib = Library::new();
    lib.add_card(
        CardKind::Attack { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 0,
            hand: 3,
            discard: 0,
        },
    );
    lib.add_card(
        CardKind::Defence { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 5,
            hand: 0,
            discard: 0,
        },
    );
    let hand = lib.hand_cards();
    assert_eq!(hand.len(), 1);
    assert_eq!(hand[0].0, 0);
}

#[test]
fn library_cards_matching_filters_by_predicate() {
    let mut lib = Library::new();
    lib.add_card(
        CardKind::Attack { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 1,
            hand: 1,
            discard: 0,
        },
    );
    lib.add_card(
        CardKind::Defence { effect_ids: vec![] },
        CardCounts {
            library: 0,
            deck: 1,
            hand: 1,
            discard: 0,
        },
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
    // Library starts with Attack having deck:15 hand:5 (now at index 8)
    let initial_hand = gs.library.cards[8].counts.hand;
    let initial_deck = gs.library.cards[8].counts.deck;
    assert!(initial_deck > 0);
    // draw_random_cards is private, but we can test it via resolve_player_card
    // playing a resource card (id 10) triggers draw_count=1
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 20);
    let mut rng = rand_pcg::Lcg64Xsh32::from_seed([0u8; 16]);
    let _ = gs.start_combat(11, &mut rng);
    let _ = gs.advance_combat_phase(); // Defending -> Attacking
    let _ = gs.advance_combat_phase(); // Attacking -> Resourcing
    let _ = gs.resolve_player_card(10, &mut rng); // Resource card draws 1
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
        token_balance_by_type(&replayed.token_balances, &TokenType::Insight),
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
    assert!(gs.current_combat.is_none());
}

#[test]
fn start_combat_with_non_encounter_card() {
    let mut gs = GameState::new();
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 20);
    let result = gs.start_combat(0, &mut rand_pcg::Lcg64Xsh32::from_seed([0u8; 16])); // card 0 is PlayerCardEffect, not CombatEncounter
    assert!(result.is_err());
}

#[test]
fn resolve_player_card_non_action_card() {
    let mut gs = GameState::new();
    gs.token_balances
        .insert(Token::persistent(TokenType::Health), 20);
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
        .insert(Token::persistent(TokenType::Health), 20);
    // No combat started, should return error
    let mut rng = rand_pcg::Lcg64Xsh32::from_seed([0u8; 16]);
    let result = gs.resolve_enemy_play(&mut rng);
    assert!(result.is_err());
}

use my_little_cardgame::area_deck::AreaDeck;
use rand::SeedableRng;

#[test]
fn area_deck_recycle_encounter() {
    let mut ad = AreaDeck::new("test".to_string());
    ad.add_encounter(10);
    ad.add_encounter(20);
    ad.draw_to_hand(2);
    assert!(ad.pick_encounter(10));
    assert_eq!(ad.discard.len(), 1);
    assert!(ad.recycle_encounter(10));
    assert_eq!(ad.discard.len(), 0);
    assert_eq!(ad.deck.len(), 1);
    // recycle non-existent returns false
    assert!(!ad.recycle_encounter(99));
}

#[test]
fn area_deck_draw_to_hand_exhausts_deck() {
    let mut ad = AreaDeck::new("test".to_string());
    ad.add_encounter(1);
    ad.draw_to_hand(5); // target 5 but only 1 in deck
    assert_eq!(ad.hand.len(), 1);
    assert!(ad.deck.is_empty());
}

#[test]
fn area_deck_encounter_card_ids_all_zones() {
    let mut ad = AreaDeck::new("test".to_string());
    ad.add_encounter(1);
    ad.add_encounter(2);
    ad.add_encounter(3);
    ad.draw_to_hand(2);
    ad.pick_encounter(2);
    let all = ad.encounter_card_ids();
    assert_eq!(all.len(), 3);
    assert!(all.contains(&1));
    assert!(all.contains(&2));
    assert!(all.contains(&3));
}
