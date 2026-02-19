use crate::deck::card::get_card;
use crate::deck::token::{Token, TokenPermanence};
use crate::player_data::PlayerData;
use rocket::State;

pub async fn resolve_card_effects(
    card_id: usize,
    owner_is_player: bool,
    player_data: &State<PlayerData>,
) {
    // Lookup card
    if let Some(card) = get_card(card_id, player_data).await {
        apply_effects(&card.effects, owner_is_player, player_data).await;
    }
}

pub async fn apply_effects(
    effects: &[Token],
    owner_is_player: bool,
    player_data: &State<PlayerData>,
) {
    use crate::deck::token::TokenType as TT;

    // Local operation types to apply after building them under the combat lock
    #[derive(Debug)]
    enum Op {
        AddPlayerToken(Token),
        DamagePlayer(u32),
        /// Set enemy token counts (None means leave unchanged)
        SetEnemyTokenCounts {
            enemy_idx: usize,
            health: Option<u32>,
            dodge: Option<u32>,
        },
    }

    let mut ops: Vec<Op> = Vec::new();

    // Snapshot and prepare operations while holding the combat lock, but do not await other locks here.
    {
        let mut combat_lock = player_data.current_combat.lock().await;
        let combat_option: &mut Option<crate::combat::Combat> = combat_lock.as_mut();
        if combat_option.is_none() {
            return;
        }

        let combat = match combat_option.as_mut() {
            Some(c) => c,
            None => return,
        };

        if owner_is_player {
            if combat.enemies.is_empty() {
                return;
            }

            // Work on the first enemy deterministically
            let enemy = &mut combat.enemies[0];

            // Snapshot current dodge and health counters
            let mut dodge_count: u32 = enemy
                .tokens
                .iter()
                .find(|t| t.token_type == TT::Dodge)
                .map(|t| t.count)
                .unwrap_or(0);
            let mut health_count: u32 = enemy
                .tokens
                .iter()
                .find(|t| t.token_type == TT::Health)
                .map(|t| t.count)
                .unwrap_or(0);

            for token in effects.iter() {
                match token.token_type {
                    TT::Dodge => {
                        ops.push(Op::AddPlayerToken(token.clone()));
                    }
                    TT::Stamina | TT::Mana => {
                        ops.push(Op::AddPlayerToken(token.clone()));
                    }
                    TT::Health => {
                        // Damage to enemy: consume enemy dodge first (snapshot), then reduce health
                        let used = std::cmp::min(dodge_count, token.count);
                        dodge_count = dodge_count.saturating_sub(used);
                        let remaining = token.count.saturating_sub(used);
                        if remaining > 0 {
                            health_count = health_count.saturating_sub(remaining);
                        }
                    }
                    _ => {}
                }
            }

            // Record final enemy counts to set later
            ops.push(Op::SetEnemyTokenCounts {
                enemy_idx: 0,
                health: Some(health_count),
                dodge: Some(dodge_count),
            });
        } else {
            // owner is enemy; target player for damage, but enemy tokens (dodge) are stored on enemy in combat
            if combat.enemies.is_empty() {
                return;
            }
            let enemy = &mut combat.enemies[0];

            // Snapshot enemy dodge
            let mut dodge_count: u32 = enemy
                .tokens
                .iter()
                .find(|t| t.token_type == TT::Dodge)
                .map(|t| t.count)
                .unwrap_or(0);

            for token in effects.iter() {
                match token.token_type {
                    TT::Dodge => {
                        dodge_count = dodge_count.saturating_add(token.count);
                    }
                    TT::Stamina | TT::Mana => {
                        // ignore for now
                    }
                    TT::Health => {
                        // record player damage to apply after dropping combat lock
                        ops.push(Op::DamagePlayer(token.count));
                    }
                    _ => {}
                }
            }

            // Record final enemy dodge count
            ops.push(Op::SetEnemyTokenCounts {
                enemy_idx: 0,
                health: None,
                dodge: Some(dodge_count),
            });
        }
    }

    // Apply player-targeting ops (add tokens, damage player) while holding player tokens lock
    {
        let mut player_tokens = player_data.tokens.lock().await;
        for op in ops.iter() {
            match op {
                Op::AddPlayerToken(token) => {
                    let existing = player_tokens
                        .iter_mut()
                        .find(|t| t.token_type == token.token_type);
                    if let Some(t) = existing {
                        t.count += token.count;
                    } else {
                        player_tokens.push(Token {
                            token_type: token.token_type.clone(),
                            permanence: token.permanence.clone(),
                            count: token.count,
                        });
                    }
                }
                Op::DamagePlayer(dmg) => {
                    let mut remaining = *dmg as i32;
                    let mut dodge_opt =
                        player_tokens.iter_mut().find(|t| t.token_type == TT::Dodge);
                    if let Some(dodge) = dodge_opt.as_mut() {
                        let used = std::cmp::min(dodge.count, *dmg);
                        dodge.count -= used;
                        remaining -= used as i32;
                    }
                    if remaining > 0 {
                        let mut health_opt = player_tokens
                            .iter_mut()
                            .find(|t| t.token_type == TT::Health);
                        if let Some(h) = health_opt.as_mut() {
                            let new_count = (h.count as i32 - remaining) as u32;
                            h.count = new_count;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Apply enemy-targeting ops (set token counts) under the combat lock
    {
        let mut combat_lock = player_data.current_combat.lock().await;
        let combat_option: &mut Option<crate::combat::Combat> = combat_lock.as_mut();
        if combat_option.is_none() {
            // Combat already ended concurrently
            return;
        }
        let combat = match combat_option.as_mut() {
            Some(c) => c,
            None => return,
        };

        for op in ops.iter() {
            if let Op::SetEnemyTokenCounts {
                enemy_idx,
                health,
                dodge,
            } = op
            {
                if *enemy_idx < combat.enemies.len() {
                    let enemy = &mut combat.enemies[*enemy_idx];
                    if let Some(d) = *dodge {
                        let existing = enemy.tokens.iter_mut().find(|t| t.token_type == TT::Dodge);
                        if let Some(t) = existing {
                            t.count = d;
                        } else {
                            enemy.tokens.push(Token {
                                token_type: TT::Dodge,
                                permanence: TokenPermanence::UsedOnUnit,
                                count: d,
                            });
                        }
                    }
                    if let Some(h) = *health {
                        let existing = enemy.tokens.iter_mut().find(|t| t.token_type == TT::Health);
                        if let Some(t) = existing {
                            t.count = h;
                        } else {
                            enemy.tokens.push(Token {
                                token_type: TT::Health,
                                permanence: TokenPermanence::UsedOnUnit,
                                count: h,
                            });
                        }
                    }
                }
            }
        }
    }

    // Determine if combat ended and record result if so
    let mut end_combat = false;
    // compute player health
    let player_health = {
        let tokens = player_data.tokens.lock().await;
        tokens
            .iter()
            .find(|t| t.token_type == TT::Health)
            .map(|t| t.count as i32)
            .unwrap_or(0)
    };
    // compute enemy health
    let enemy_health = {
        let combat_opt = player_data.current_combat.lock().await.clone();
        if let Some(combat_box) = combat_opt.as_ref() {
            if !combat_box.enemies.is_empty() {
                let mut eh = 0i32;
                for t in combat_box.enemies[0].tokens.iter() {
                    if t.token_type == TT::Health {
                        eh = t.count as i32;
                        break;
                    }
                }
                eh
            } else {
                0
            }
        } else {
            0
        }
    };

    if enemy_health <= 0 || player_health <= 0 {
        end_combat = true;
    }

    if end_combat {
        let player_health = {
            let tokens = player_data.tokens.lock().await;
            tokens
                .iter()
                .find(|t| t.token_type == TT::Health)
                .map(|t| t.count as i32)
                .unwrap_or(0)
        };
        let enemy_health = {
            let combat_opt = player_data.current_combat.lock().await.clone();
            if let Some(combat_box) = combat_opt.as_ref() {
                if !combat_box.enemies.is_empty() {
                    let mut eh = 0i32;
                    for t in combat_box.enemies[0].tokens.iter() {
                        if t.token_type == TT::Health {
                            eh = t.count as i32;
                            break;
                        }
                    }
                    eh
                } else {
                    0
                }
            } else {
                0
            }
        };
        let winner = if enemy_health <= 0 && player_health > 0 {
            "Player"
        } else if player_health <= 0 && enemy_health > 0 {
            "Enemy"
        } else {
            "Draw"
        };
        // store result
        let mut last = player_data.last_combat_result.lock().await;
        *last = Some(crate::combat::CombatResult {
            winner: winner.to_string(),
        });

        // drop the combat by setting it to None
        let mut combat_lock = player_data.current_combat.lock().await;
        let combat_option: &mut Option<crate::combat::Combat> = combat_lock.as_mut();
        *combat_option = None;
    }
}
