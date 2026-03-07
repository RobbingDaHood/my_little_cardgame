use crate::library::types::{
    self, CardCounts, CardKind, Discipline, EncounterKind, EncounterOutcome, EncounterState,
    ResearchCandidate, ResearchEncounterState,
};
use crate::library::{GameState, Library};
use rand::RngCore;

use crate::library::game_state::roll_concrete_effect;

pub(crate) fn register_research_cards(lib: &mut Library, rng: &mut rand_pcg::Lcg64Xsh32) {
    lib.add_card(
        CardKind::Encounter {
            encounter_kind: EncounterKind::Research {
                research_def: types::ResearchDef {},
            },
        },
        CardCounts {
            library: 0,
            deck: 1,
            hand: 0,
            discard: 0,
        },
        rng,
        vec![Discipline::Research],
    );
}

impl GameState {
    pub fn start_research_encounter(&mut self, encounter_card_id: usize) -> Result<(), String> {
        if self.current_encounter.is_some() {
            return Err("Already in an encounter".to_string());
        }

        let card = self
            .library
            .get(encounter_card_id)
            .ok_or("Encounter card not found")?
            .clone();
        match &card.kind {
            CardKind::Encounter {
                encounter_kind: EncounterKind::Research { .. },
            } => {}
            _ => return Err("Card is not a research encounter".to_string()),
        }

        self.current_encounter = Some(EncounterState::Research(ResearchEncounterState {
            encounter_card_id,
            outcome: EncounterOutcome::Undecided,
            candidates: None,
        }));
        self.encounter_phase = types::EncounterPhase::InEncounter;
        Ok(())
    }

    pub fn research_choose_project(
        &mut self,
        discipline: Discipline,
        tier_count: u32,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Research(r)) if r.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active research encounter".to_string()),
        }

        if tier_count == 0 {
            return Err("Tier count must be at least 1".to_string());
        }

        // Cost: 10 * 2^(tier_count - 1)
        let insight_cost = 10_i64 * (1i64 << (tier_count - 1));

        let insight_key = types::Token::persistent(types::TokenType::Insight);
        let balance = self.token_balances.get(&insight_key).copied().unwrap_or(0);
        if balance < insight_cost {
            return Err(format!(
                "Insufficient Insight: need {} but have {}",
                insight_cost, balance
            ));
        }

        let matching_effects = self.library.card_effects_for_discipline(&discipline);
        if matching_effects.is_empty() {
            return Err(format!(
                "No card effects available for discipline {:?}",
                discipline
            ));
        }

        *self.token_balances.entry(insight_key).or_insert(0) -= insight_cost;

        let effect_ids: Vec<usize> = matching_effects.iter().map(|(id, _)| *id).collect();

        let mut candidates = Vec::with_capacity(3);
        for _ in 0..3 {
            let mut effects = Vec::with_capacity(tier_count as usize);
            for _ in 0..tier_count {
                let idx = (rng.next_u64() as usize) % effect_ids.len();
                let effect_id = effect_ids[idx];
                let concrete = roll_concrete_effect(rng, effect_id, &self.library);
                effects.push(concrete);
            }
            candidates.push(ResearchCandidate {
                discipline: discipline.clone(),
                effects,
                tier_count,
            });
        }

        if let Some(EncounterState::Research(r)) = &mut self.current_encounter {
            r.candidates = Some(candidates);
        }

        Ok(())
    }

    pub fn research_select_candidate(&mut self, candidate_index: usize) -> Result<(), String> {
        let candidates = match &self.current_encounter {
            Some(EncounterState::Research(r))
                if r.outcome == EncounterOutcome::Undecided && r.candidates.is_some() =>
            {
                r.candidates.as_ref().ok_or("No candidates generated")?
            }
            _ => return Err("No active research encounter with candidates".to_string()),
        };

        if candidate_index >= candidates.len() {
            return Err(format!(
                "Invalid candidate index {}, must be 0-{}",
                candidate_index,
                candidates.len() - 1
            ));
        }

        let chosen = candidates[candidate_index].clone();
        let tier_count = chosen.tier_count;
        // Total research cost: 20 * 2^(tier_count - 1)
        let total_cost = 20_i64 * (1i64 << (tier_count - 1));

        self.current_research = Some(types::ResearchProject {
            discipline: chosen.discipline.clone(),
            tier_count,
            chosen_card: chosen,
            progress: 0,
            total_cost,
        });

        if let Some(EncounterState::Research(r)) = &mut self.current_encounter {
            r.candidates = None;
        }

        Ok(())
    }

    pub fn research_progress(
        &mut self,
        amount: i64,
        rng: &mut rand_pcg::Lcg64Xsh32,
    ) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Research(r)) if r.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active research encounter".to_string()),
        }

        let project = self
            .current_research
            .as_ref()
            .ok_or("No active research project")?;

        if amount <= 0 {
            return Err("Progress amount must be positive".to_string());
        }

        let remaining_cost = project.total_cost - project.progress;
        // Cap at 33% of total cost
        let max_per_action = (project.total_cost + 2) / 3;
        let insight_key = types::Token::persistent(types::TokenType::Insight);
        let available = self.token_balances.get(&insight_key).copied().unwrap_or(0);

        let actual = amount
            .min(max_per_action)
            .min(available)
            .min(remaining_cost);
        if actual <= 0 {
            return Err(
                "Cannot make progress: insufficient Insight or already complete".to_string(),
            );
        }

        *self.token_balances.entry(insight_key).or_insert(0) -= actual;

        let project = self
            .current_research
            .as_mut()
            .ok_or("No active research project")?;
        project.progress += actual;

        if project.progress >= project.total_cost {
            let finished = self
                .current_research
                .take()
                .ok_or("Research project disappeared")?;

            self.library.add_card(
                CardKind::Attack {
                    effects: finished.chosen_card.effects.clone(),
                },
                CardCounts {
                    library: 0,
                    deck: 0,
                    hand: 0,
                    discard: 0,
                },
                rng,
                vec![finished.discipline.clone()],
            );
        }

        Ok(())
    }

    pub fn conclude_research_encounter(&mut self) -> Result<(), String> {
        match &self.current_encounter {
            Some(EncounterState::Research(r)) if r.outcome == EncounterOutcome::Undecided => {}
            _ => return Err("No active research encounter to conclude".to_string()),
        }
        self.finish_research_encounter();
        Ok(())
    }

    pub fn abort_research_encounter(&mut self) {
        self.finish_research_encounter();
    }

    fn finish_research_encounter(&mut self) {
        self.last_encounter_result = Some(EncounterOutcome::PlayerWon);
        self.encounter_results.push(EncounterOutcome::PlayerWon);
        self.current_encounter = None;
        self.encounter_phase = types::EncounterPhase::Scouting;
    }
}
