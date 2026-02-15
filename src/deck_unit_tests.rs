#[cfg(test)]
mod tests {
    use crate::deck::{Deck, DeckCard, CardState};
    use crate::deck::card::CardType;
    use std::collections::HashMap;
    use rand_pcg::Lcg64Xsh32;
    use rand::SeedableRng;

    #[test]
    fn test_change_card_state_success_and_failure() {
        // Successful state change
        let mut deck = Deck {
            cards: vec![DeckCard { id: 5, state: HashMap::from([(CardState::Deck, 1)]) }],
            id: 1,
            contains_card_types: vec![CardType::Defence],
        };
        let ok = deck.change_card_state(5, CardState::Hand, CardState::Deck);
        assert!(ok.is_ok());
        let card = deck.cards.iter().find(|c| c.id == 5).expect("card exists");
        assert_eq!(card.state.get(&CardState::Hand).cloned().unwrap_or(0), 1);

        // Failure when card not present
        let mut other = Deck { cards: vec![], id: 2, contains_card_types: vec![CardType::Attack] };
        let err = other.change_card_state(99, CardState::Hand, CardState::Deck);
        assert!(err.is_err());
    }

    #[test]
    fn test_draw_and_change_random_cards_state_runs() {
        // Build a deck with multiple cards so draw_cards can run
        let mut deck = Deck {
            cards: vec![
                DeckCard { id: 1, state: HashMap::from([(CardState::Deck, 2)]) },
                DeckCard { id: 2, state: HashMap::from([(CardState::Deck, 2)]) },
            ],
            id: 3,
            contains_card_types: vec![CardType::Attack],
        };
        let mut rng = Lcg64Xsh32::from_seed([2u8; 16]);
        // draw one card (should move one Deck -> Hand)
        let _ = deck.draw_cards(1, &mut rng);
        // ensure some card now has Hand state
        let has_hand = deck.cards.iter().any(|c| c.state.get(&CardState::Hand).is_some());
        assert!(has_hand);
    }
}
