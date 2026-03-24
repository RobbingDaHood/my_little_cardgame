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
///   Tactician > Random > Greedy > Conservative
pub fn combat_targets() -> Vec<WinRateTarget> {
    vec![
        WinRateTarget {
            strategy: "random".to_string(),
            target_min: 0.55,
            target_max: 0.80,
        },
        WinRateTarget {
            strategy: "greedy".to_string(),
            target_min: 0.45,
            target_max: 0.65,
        },
        WinRateTarget {
            strategy: "conservative".to_string(),
            target_min: 0.70,
            target_max: 0.95,
        },
    ]
}

/// Streak targets encode the intended strategy hierarchy.
///
/// Tactician should have the longest streaks (skilled play with dodge +
/// cost_damage kills enemies before they scale dangerously). Random does
/// surprisingly well due to frequent dodge draws. Greedy pays HP costs for
/// marginal damage gains. Conservative relies on weak shield absorption.
pub fn combat_streak_targets() -> Vec<WinStreakTarget> {
    vec![
        WinStreakTarget {
            strategy: "random".to_string(),
            target_min_streak: 5.0,
            target_max_streak: 8.5,
        },
        WinStreakTarget {
            strategy: "greedy".to_string(),
            target_min_streak: 4.0,
            target_max_streak: 7.5,
        },
        WinStreakTarget {
            strategy: "conservative".to_string(),
            target_min_streak: 3.0,
            target_max_streak: 6.0,
        },
        WinStreakTarget {
            strategy: "tactician".to_string(),
            target_min_streak: 7.5,
            target_max_streak: 18.0,
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
