# 0003 - Deck edge-case tests

Summary:
Add unit tests around Deck::change_card_state and Deck::change_random_cards_state for boundary conditions (empty deck, zero-count source state, multiple draws, etc.).

Tasks:
- Add tests that assert proper errors when attempting to move a card whose source state has count == 0.
- Add tests for drawing from empty decks and ensure errors are surfaced cleanly.
- Update coverage target if needed.
