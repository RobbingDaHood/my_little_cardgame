When the below point states "Roadmap" it means edit the roadmap.md directly.

1. ✅ Roadmap change: Step 9.5 Better mining 
    1. Each player card effect that gives light level: 
        1. Have a max cap like any other positive gain card effect. 
            1. Like all the other gain cards then it rolls the cap first and then have the gain as a pecentage from that. 
        1. It also has a cost percentage based on the gain: the cost is in a small amount of wood tokens. 
- ✅ **Extend HasDeckCounts to player library cards:** `LibraryCard` uses `CardCounts` (with an extra `library` field) instead of `DeckCounts`. Consider a broader `HasCounts` trait hierarchy or unifying `CardCounts` and `DeckCounts` so player deck draw/shuffle operations can also use generic functions, further reducing duplication in `draw_player_cards_of_kind`.
- ✅ **Generalize ore play-random in mining.rs:** `resolve_ore_play` still has inline logic for picking a random ore card from hand and moving it to discard. Refactor to use `deck_play_random`, matching how combat's `resolve_enemy_play` and fishing's `fish_play_random` were updated.

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
