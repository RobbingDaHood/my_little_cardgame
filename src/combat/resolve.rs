/// Resolve a card play during combat using the unified library model.
///
/// This module delegates to GameState's combat resolution methods.
/// The old Token-based effect system has been replaced by CardEffect
/// from the Library.
pub async fn resolve_card_effects(
    card_id: usize,
    owner_is_player: bool,
    game_state: &std::sync::Arc<rocket::futures::lock::Mutex<crate::library::GameState>>,
    player_data: &rocket::State<crate::player_data::PlayerData>,
) {
    let mut gs = game_state.lock().await;
    if owner_is_player {
        let _ = gs.resolve_player_card(card_id);
    } else {
        let mut rng = player_data.random_generator_state.lock().await;
        let _ = gs.resolve_enemy_play(&mut rng);
    }
}
