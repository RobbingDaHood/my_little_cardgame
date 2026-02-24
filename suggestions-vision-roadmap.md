# Suggestions for vision.md and roadmap.md

All suggestions from the post-7.7 implementation have been applied to vision.md and roadmap.md.

---

## Resolved contradictions and clarifications

1. **Enemy resource draws**: Resource card draws apply to the same deck type only. When a resource card triggers DrawCards, it draws cards for **each of the three deck types** (attack, defence, resource), so all deck types have a replenishment mechanism.

2. **CardEffect deck future usage**: The EnemyCardEffect deck will be used during the post-encounter scouting phase to help generate new encounters for the encounter deck. The PlayerCardEffect deck will be used during research to help generate new cards for the library. Detailed mechanics will be fleshed out in later roadmap steps.

3. **Card ID stability**: New cards must always be appended to the end of the Library vector, never inserted at the beginning, to preserve stable card IDs.

4. **DrawCards as TokenType**: DrawCards is an effect-only trigger, not a real token stored in balances. A future refactor should separate it from the TokenType enum into a proper CardEffect mechanism.
