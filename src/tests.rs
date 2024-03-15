#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use rocket::http::{Header, Status};
    use rocket::http::uncased::Uncased;
    use rocket::local::blocking::Client;
    use rocket::serde::json::serde_json;

    use crate::deck::{Card, CardState, CreateDeck, Deck, DeckCard, rocket_uri_macro_list_all_decks};
    use crate::deck::card::{CardCreate, CardType, rocket_uri_macro_list_all_cards};
    use crate::deck::card::rocket_uri_macro_create_card;
    use crate::deck::rocket_uri_macro_add_card_to_deck;
    use crate::deck::rocket_uri_macro_create_deck;
    use crate::deck::token::{PermanentDefinition, Token, TokenPermanence, TokenType};
    use crate::rocket_initialize;
    use crate::status_messages::new_status;
    use crate::status_messages::Status as MyStatus;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

        let list_of_cards = get_cards(&client);
        assert_eq!(1, list_of_cards.len());

        let new_card_attack = getAttackCard();
        let location_header_card_attack = post_card(&client, &new_card_attack);
        assert_eq!("/cards/6", location_header_card_attack);

        let new_card_ressource = getRessourceCard();
        let location_header_card_ressource = post_card(&client, &new_card_ressource);
        assert_eq!("/cards/7", location_header_card_ressource);

        let card_id_attack = get_card(&client, new_card_attack, location_header_card_attack);
        let card_id_ressource = get_card(&client, new_card_ressource, location_header_card_ressource);

        get_decks(&client, 1);

        let location_header_deck = post_deck(&client);
        assert_eq!("/decks/1", location_header_deck);

        get_decks(&client, 2);

        let created_deck = get_deck(&client, location_header_deck.clone());
        assert_eq!(0, created_deck.cards.len());

        let deck_card = DeckCard {
            id: card_id_attack,
            state: HashMap::from([(CardState::Deck, 20)]),
        };
        let location_header_card_in_deck = post_card_to_deck(&client, created_deck.id, deck_card);
        assert_eq!("/decks/1/cards/6", location_header_card_in_deck);

        let deck_card = DeckCard {
            id: card_id_ressource,
            state: HashMap::from([(CardState::Deck, 20)]),
        };
        post_card_to_deck_fail_on_type(&client, created_deck.id, deck_card,
                                       "Card with id 7 is of type Ressource and that is not part of the types '[Attack]' allowed in deck with id 1");

        let card_in_deck = get_card_in_deck(&client, location_header_card_in_deck.clone());
        assert_eq!(card_in_deck.id, card_id_attack);
        assert_eq!(*card_in_deck.state.get(&CardState::Deck).unwrap(), 20);

        let created_deck = get_deck(&client, location_header_deck.clone());
        assert_eq!(1, created_deck.cards.len());
        assert_eq!(0, created_deck.cards.iter()
            .filter(|card| card.state.get(&CardState::Deleted).is_some())
            .count());

        delete_card_in_deck(&client, location_header_card_in_deck);

        let created_deck = get_deck(&client, location_header_deck);
        assert_eq!(1, created_deck.cards.len());
        assert_eq!(1, created_deck.cards.iter()
            .filter(|card| card.state.get(&CardState::Deleted).is_some())
            .count());
    }

    fn getRessourceCard() -> CardCreate {
        CardCreate {
            card_type_id: 1,
            card_type: CardType::Ressource,
            effects: vec![
                Token {
                    token_type: TokenType::Health,
                    count: 1,
                    permanence: TokenPermanence::Permanent(
                        PermanentDefinition {
                            max_count: 20,
                        }
                    ),
                }
            ],
            costs: vec![
                Token {
                    token_type: TokenType::Mana,
                    count: 1,
                    permanence: TokenPermanence::Permanent(
                        PermanentDefinition {
                            max_count: 20,
                        }
                    ),
                }
            ],
            count: 22,
        }
    }

    fn getAttackCard() -> CardCreate {
        CardCreate {
            card_type_id: 1,
            card_type: CardType::Attack,
            effects: vec![
                Token {
                    token_type: TokenType::Health,
                    count: 1,
                    permanence: TokenPermanence::Permanent(
                        PermanentDefinition {
                            max_count: 20,
                        }
                    ),
                }
            ],
            costs: vec![
                Token {
                    token_type: TokenType::Mana,
                    count: 1,
                    permanence: TokenPermanence::Permanent(
                        PermanentDefinition {
                            max_count: 20,
                        }
                    ),
                }
            ],
            count: 22,
        }
    }

    fn get_decks(client: &Client, expected_number_of_decks: usize) {
        let response = client.get(uri!(list_all_decks)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let string_body = response.into_string().unwrap();
        let list_of_decks: Vec<Deck> = serde_json::from_str(string_body.as_str()).unwrap();
        assert_eq!(expected_number_of_decks, list_of_decks.len());
    }

    fn get_card(client: &Client, new_card: CardCreate, location_header: String) -> usize {
        let response = client.get(location_header).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let string_body = response.into_string().unwrap();
        let created_card: Card = serde_json::from_str(string_body.as_str()).unwrap();
        created_card.id
    }

    fn get_deck(client: &Client, location_header: String) -> Deck {
        let response = client.get(location_header).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let string_body = response.into_string().unwrap();
        let created_deck: Deck = serde_json::from_str(string_body.as_str()).unwrap();
        created_deck
    }

    fn delete_card_in_deck(client: &Client, location_header: String) {
        let response = client.delete(location_header).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }

    fn get_card_in_deck(client: &Client, location_header: String) -> DeckCard {
        let response = client.get(location_header).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let string_body = response.into_string().unwrap();
        let created_deck: DeckCard = serde_json::from_str(string_body.as_str()).unwrap();
        created_deck
    }

    fn post_card(client: &Client, new_card: &CardCreate) -> String {
        let body_json = serde_json::to_string(&new_card).unwrap();
        let response = client.post(uri!(create_card))
            .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
            .body(body_json)
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        let response_headers = response.headers();
        let location_header_list: Vec<_> = response_headers.get("location").collect();
        assert_eq!(1, location_header_list.len());
        let location_header = location_header_list.get(0).unwrap();
        location_header.to_string()
    }

    fn post_card_to_deck(client: &Client, id: usize, new_card: DeckCard) -> String {
        let body_json = serde_json::to_string(&new_card).unwrap();
        let response = client.post(uri!(add_card_to_deck(id)))
            .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
            .body(body_json)
            .dispatch();
        assert_eq!(Status::Created, response.status());
        let response_headers = response.headers();
        let location_header_list: Vec<_> = response_headers.get("location").collect();
        assert_eq!(1, location_header_list.len());
        let location_header = location_header_list.get(0).unwrap();
        location_header.to_string()
    }

    fn post_card_to_deck_fail_on_type(client: &Client, id: usize, new_card: DeckCard, expected_error_message: &str) {
        let body_json = serde_json::to_string(&new_card).unwrap();
        let response = client.post(uri!(add_card_to_deck(id)))
            .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
            .body(body_json)
            .dispatch();
        assert_eq!(Status::BadRequest, response.status());
        let body_json = response.into_string().unwrap();
        let body: MyStatus = serde_json::from_str(body_json.as_str()).unwrap();
        let expected_status: MyStatus = new_status(expected_error_message.to_string()).0;
        assert_eq!(expected_status, body);
    }

    fn post_deck(client: &Client) -> String {
        let create_deck = CreateDeck {
            contains_card_types: vec![CardType::Attack],
        };
        let create_deck_json = serde_json::to_string(&create_deck).unwrap();
        let response = client.post(uri!(create_deck))
            .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
            .body(create_deck_json)
            .dispatch();
        assert_eq!(Status::Created, response.status());
        let response_headers = response.headers();
        let location_header_list: Vec<_> = response_headers.get("location").collect();
        assert_eq!(1, location_header_list.len());
        let location_header = location_header_list.get(0).unwrap();
        location_header.to_string()
    }

    fn get_cards(client: &Client) -> Vec<Card> {
        let response = client.get(uri!(list_all_cards)).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let string_body = response.into_string().unwrap();
        let list_of_decks: Vec<Card> = serde_json::from_str(string_body.as_str()).unwrap();
        list_of_decks
    }
}
