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

/// Combat balance assertions from vision.md (±10%).
pub fn combat_targets() -> Vec<WinRateTarget> {
    vec![
        WinRateTarget {
            strategy: "random".to_string(),
            target_min: 0.20,
            target_max: 0.40,
        },
        WinRateTarget {
            strategy: "greedy".to_string(),
            target_min: 0.40,
            target_max: 0.60,
        },
        WinRateTarget {
            strategy: "conservative".to_string(),
            target_min: 0.30,
            target_max: 0.50,
        },
    ]
}

/// Streak targets: simple strategies ~3-5, enemy-aware ~10+.
pub fn combat_streak_targets() -> Vec<WinStreakTarget> {
    vec![
        WinStreakTarget {
            strategy: "random".to_string(),
            target_min_streak: 3.5,
            target_max_streak: 8.0,
        },
        WinStreakTarget {
            strategy: "greedy".to_string(),
            target_min_streak: 3.0,
            target_max_streak: 7.0,
        },
        WinStreakTarget {
            strategy: "conservative".to_string(),
            target_min_streak: 2.5,
            target_max_streak: 6.0,
        },
        WinStreakTarget {
            strategy: "tactician".to_string(),
            target_min_streak: 8.0,
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
