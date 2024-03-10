#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use rocket::http::{Header, Status};
    use rocket::http::uncased::Uncased;
    use rocket::local::blocking::Client;
    use rocket::serde::json::serde_json;

    use crate::deck::{Card, CardState, Deck, DeckCard, rocket_uri_macro_list_all_decks};
    use crate::deck::card::{CardCreate, rocket_uri_macro_list_all_cards};
    use crate::deck::card::rocket_uri_macro_create_card;
    use crate::deck::rocket_uri_macro_add_card_to_deck;
    use crate::deck::rocket_uri_macro_create_deck;
    use crate::rocket_initialize;

    #[test]
    fn hello_world() {
        let client = Client::tracked(rocket_initialize()).expect("valid rocket instance");

        let list_of_cards = get_cards(&client);
        assert_eq!(0, list_of_cards.len());

        let new_card = CardCreate {
            card_type_id: 1
        };
        let location_header_card = post_card(&client, &new_card);
        assert_eq!("/cards/0", location_header_card);

        let card_id = get_card(&client, new_card, location_header_card);

        get_decks(&client, 0);

        let location_header_deck = post_deck(&client);
        assert_eq!("/decks/0", location_header_deck);

        get_decks(&client, 1);

        let created_deck = get_deck(&client, location_header_deck.clone());
        assert_eq!(0, created_deck.cards.len());

        let deck_card = DeckCard {
            id: card_id,
            state: CardState::Deck,
        };
        let location_header_card_in_deck = post_card_to_deck(&client, created_deck.id, deck_card);
        assert_eq!("/decks/0/cards/0", location_header_card_in_deck);

        let card_in_deck = get_card_in_deck(&client, location_header_card_in_deck.clone());
        assert_eq!(card_in_deck.id, card_id);
        assert_eq!(card_in_deck.state, CardState::Deck);

        let created_deck = get_deck(&client, location_header_deck.clone());
        assert_eq!(1, created_deck.cards.len());
        assert_eq!(0, created_deck.cards.iter()
            .filter(|card| card.state == CardState::Deleted)
            .count());

        delete_card_in_deck(&client, location_header_card_in_deck);

        let created_deck = get_deck(&client, location_header_deck);
        assert_eq!(1, created_deck.cards.len());
        assert_eq!(1, created_deck.cards.iter()
            .filter(|card| card.state == CardState::Deleted)
            .count());
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
        assert_eq!(new_card.card_type_id, created_card.card_type_id);
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
        assert_eq!(response.status(), Status::Created);
        let response_headers = response.headers();
        let location_header_list: Vec<_> = response_headers.get("location").collect();
        assert_eq!(1, location_header_list.len());
        let location_header = location_header_list.get(0).unwrap();
        location_header.to_string()
    }

    fn post_deck(client: &Client) -> String {
        let response = client.post(uri!(create_deck))
            .header(Header { name: Uncased::from("Content-Type"), value: Cow::from("application/json") })
            .dispatch();
        assert_eq!(response.status(), Status::Created);
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
