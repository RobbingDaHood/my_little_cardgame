use crate::deck::card::get_card;
use crate::deck::token::{Token, TokenType, TokenPermanence};
use crate::player_data::PlayerData;
use rocket::State;

pub async fn resolve_card_effects(card_id: usize, owner_is_player: bool, player_data: &State<PlayerData>) {
    // Lookup card
    if let Some(card) = get_card(card_id, player_data).await {
        apply_effects(&card.effects, owner_is_player, player_data).await;
    }
}

pub async fn apply_effects(effects: &Vec<Token>, owner_is_player: bool, player_data: &State<PlayerData>) {
    // Acquire combat lock to find targets
    let mut combat_lock = player_data.current_combat.lock().await;
    let combat_option: &mut Option<crate::combat::Combat> = combat_lock.as_mut();
    if combat_option.is_none() {
        return;
    }

    // perform mutations inside a scoped borrow so we can decide to end combat afterwards
    let mut end_combat = false;
    {
        let combat = combat_option.as_mut().unwrap();

        // Determine target: if owner is player, target is first enemy unit; else target is player tokens
        if owner_is_player {
            if combat.enemies.is_empty() {
                return;
            }
            // target first enemy
            let enemy = &mut combat.enemies[0];
            // Apply each token effect
            for token in effects.iter() {
                match token.token_type {
                    TokenType::Dodge => {
                        // add dodge tokens to the owner (player)
                        let mut player_tokens = player_data.tokens.lock().await;
                        let existing = player_tokens.iter_mut().find(|t| t.token_type == TokenType::Dodge);
                        if let Some(t) = existing {
                            t.count += token.count;
                        } else {
                            player_tokens.push(Token { token_type: TokenType::Dodge, permanence: TokenPermanence::UsedOnUnit, count: token.count });
                        }
                    }
                    TokenType::Stamina | TokenType::Mana => {
                        let mut player_tokens = player_data.tokens.lock().await;
                        let existing = player_tokens.iter_mut().find(|t| t.token_type == token.token_type);
                        if let Some(t) = existing {
                            t.count += token.count;
                        } else {
                            player_tokens.push(Token { token_type: token.token_type.clone(), permanence: TokenPermanence::UsedOnUnit, count: token.count });
                        }
                    }
                    TokenType::Health => {
                        // interpret as damage to target
                        // First consume dodge tokens on target
                        let mut dodge_opt = enemy.tokens.iter_mut().find(|t| t.token_type == TokenType::Dodge);
                        let mut remaining = token.count as i32;
                        if let Some(dodge) = dodge_opt.as_mut() {
                            let used = std::cmp::min(dodge.count, token.count);
                            dodge.count -= used;
                            remaining -= used as i32;
                        }
                        if remaining > 0 {
                            let mut health_opt = enemy.tokens.iter_mut().find(|t| t.token_type == TokenType::Health);
                            if let Some(h) = health_opt.as_mut() {
                                let new_count = (h.count as i32 - remaining) as u32;
                                h.count = new_count;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // check if target enemy died
            let target_unit = &combat.enemies[0];
            let mut enemy_health = 0i32;
            for t in target_unit.tokens.iter() {
                if t.token_type == TokenType::Health {
                    enemy_health = t.count as i32;
                    break;
                }
            }
            if enemy_health <= 0 {
                end_combat = true;
            }
        } else {
            // owner is enemy, target is player
            for token in effects.iter() {
                match token.token_type {
                    TokenType::Dodge => {
                        // add dodge to enemy: enemy.first
                        if combat.enemies.is_empty() { continue; }
                        let enemy = &mut combat.enemies[0];
                        let existing = enemy.tokens.iter_mut().find(|t| t.token_type == TokenType::Dodge);
                        if let Some(t) = existing { t.count += token.count; } else { enemy.tokens.push(Token { token_type: TokenType::Dodge, permanence: TokenPermanence::UsedOnUnit, count: token.count }); }
                    }
                    TokenType::Stamina | TokenType::Mana => {
                        // enemy resource: ignore for now
                    }
                    TokenType::Health => {
                        // attack: damage player
                        let mut player_tokens = player_data.tokens.lock().await;
                        let mut dodge_opt = player_tokens.iter_mut().find(|t| t.token_type == TokenType::Dodge);
                        let mut remaining = token.count as i32;
                        if let Some(dodge) = dodge_opt.as_mut() {
                            let used = std::cmp::min(dodge.count, token.count);
                            dodge.count -= used;
                            remaining -= used as i32;
                        }
                        if remaining > 0 {
                            let mut health_opt = player_tokens.iter_mut().find(|t| t.token_type == TokenType::Health);
                            if let Some(h) = health_opt.as_mut() {
                                let new_count = (h.count as i32 - remaining) as u32;
                                h.count = new_count;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // check if player died
            let mut player_health = 0i32;
            for t in player_data.tokens.lock().await.iter() {
                if t.token_type == TokenType::Health {
                    player_health = t.count as i32;
                    break;
                }
            }
            if player_health <= 0 {
                end_combat = true;
            }
        }
    }

    if end_combat {
        // determine winner and store result
        let mut player_health = 0i32;
        for t in player_data.tokens.lock().await.iter() {
            if t.token_type == TokenType::Health { player_health = t.count as i32; break; }
        }
        let mut enemy_health = 0i32;
        if let Some(combat_box) = combat_option.as_ref() {
            if !combat_box.enemies.is_empty() {
                for t in combat_box.enemies[0].tokens.iter() {
                    if t.token_type == TokenType::Health { enemy_health = t.count as i32; break; }
                }
            }
        }
        let winner = if enemy_health <= 0 && player_health > 0 {
            "Player"
        } else if player_health <= 0 && enemy_health > 0 {
            "Enemy"
        } else {
            "Draw"
        };
        // store result
        let mut last = player_data.last_combat_result.lock().await;
        *last = Some(crate::combat::CombatResult { winner: winner.to_string() });

        // drop the combat by setting it to None
        *combat_option = None;
    }

}
