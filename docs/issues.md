When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Minor detail: The requirement about static ids for cards is not that documentation need to keep referring to an id: But just that when a card has been inserted in the library then it will keep that id for the rest of the game.
    1. There are quite a lot of text describing fixed ids, so you can relax a bit on that.
    2. ✅ Resolved: Documentation has been relaxed; card IDs are stable after insertion.
1. Edit this both in the code, the roadmap and vision document: If the full hand are cards that cannot be paid: then it is an autoloss for the player.
    1. There are future steps for handling enemies.
    2. ✅ Resolved in 9.3: Autoloss implemented — if all combat hand cards are unpayable, encounter ends as PlayerLost.
1. CardEffects that grant tokens and are under a cap: The cap will function a bit like the "cost" of other cards, just reversed: so high cost low cap and the reverse. So the cap is a range min-max and then the tokens gained is a percentage of that value.
    1. The token percentage is also a min-max amount.
    1. This is how all of them should work.
    2. ✅ Resolved in 9.3: cap_min/cap_max and gain_min_percent/gain_max_percent on ChangeTokens, rolled to ConcreteEffect, applied during token grant.
1. The "Milestone tokens" on combat is in plural, around 100 of them.
    1. ✅ Resolved in 9.3: MilestoneInsight token added; 100 granted per combat victory.
1. All the CardEffects that ChangeTokens should define the duration of the token: even each TokenType always has the same duration.
    1. This is because in the future I would likely expand the duration possibilities of different TokenTypes.
    2. ✅ Resolved in 9.3: duration: TokenLifecycle field added to ChangeTokens with serde default for backward compat.

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear.

Also, make general improvement to both files.
