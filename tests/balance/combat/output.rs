use serde::Serialize;

/// Win-rate target for a combat strategy.
#[derive(Debug, Clone, Serialize)]
pub struct WinRateTarget {
    pub strategy: String,
    pub target_min: f64,
    pub target_max: f64,
}

/// Win-streak target for a combat strategy.
#[derive(Debug, Clone, Serialize)]
pub struct WinStreakTarget {
    pub strategy: String,
    pub target_min_streak: f64,
    pub target_max_streak: f64,
    /// When true, assert on avg_max_win_streak instead of overall_avg_streak.
    /// Used for tier-2 strategies where peak performance matters more than average.
    pub use_max_streak: bool,
}

/// Combat balance assertions.
///
/// Win-rate targets are higher than originally envisioned (0.20-0.50) because:
/// 1. Four initial encounters at base difficulty are easy wins for all strategies.
/// 2. Scouting mutation only scales ~10% of enemy card effects per step while
///    enemy HP scales fully — encounters become HP sponges, not death threats.
/// 3. After death the player resets to full HP but difficulty stays high, yet
///    they still win 1-3 encounters before the death spiral catches up.
/// 4. Percentage-based cost_damage never outright kills the player.
///
/// The streak hierarchy is the primary balance signal:
///   Tactician tier (greedy/conservative) > Simple tier (Random/Greedy/Conservative)
pub fn combat_targets() -> Vec<WinRateTarget> {
    vec![
        WinRateTarget {
            strategy: "random".to_string(),
            target_min: 0.50,
            target_max: 0.95,
        },
        WinRateTarget {
            strategy: "greedy".to_string(),
            target_min: 0.40,
            target_max: 0.99,
        },
        WinRateTarget {
            strategy: "conservative".to_string(),
            target_min: 0.70,
            target_max: 0.99,
        },
    ]
}

/// Streak targets encode the intended strategy hierarchy.
///
/// Tactician variants should always outperform simple strategies.
/// Tactician-greedy uses stamina-cost attack cards aggressively to kill
/// enemies fast. Tactician-conservative conserves resources and uses
/// stamina-cost dodge for massive damage absorption.
pub fn combat_streak_targets() -> Vec<WinStreakTarget> {
    vec![
        WinStreakTarget {
            strategy: "random".to_string(),
            target_min_streak: 3.0,
            target_max_streak: 10.0,
            use_max_streak: false,
        },
        WinStreakTarget {
            strategy: "greedy".to_string(),
            target_min_streak: 3.0,
            target_max_streak: 10.0,
            use_max_streak: false,
        },
        WinStreakTarget {
            strategy: "conservative".to_string(),
            target_min_streak: 2.5,
            target_max_streak: 7.0,
            use_max_streak: false,
        },
        WinStreakTarget {
            strategy: "tactician_greedy".to_string(),
            target_min_streak: 8.0,
            target_max_streak: 18.0,
            use_max_streak: true,
        },
        WinStreakTarget {
            strategy: "tactician_conservative".to_string(),
            target_min_streak: 8.0,
            target_max_streak: 18.0,
            use_max_streak: true,
        },
    ]
}

/// Combat-specific metrics for a strategy's performance.
#[derive(Debug, Serialize)]
pub struct CombatReport {
    pub total_encounters: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
    pub target_min: f64,
    pub target_max: f64,
    pub pass: bool,
    pub avg_rounds_per_encounter: f64,
    pub avg_max_win_streak: f64,
    pub overall_avg_streak: f64,
    pub streak_target_min: f64,
    pub streak_target_max: f64,
    pub streak_pass: bool,
    pub rounds_pass: bool,
}
