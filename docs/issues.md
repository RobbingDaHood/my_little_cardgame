When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Minor detail: The requirement about statuc ids for cards is not that documentation need to keep reffering to a id: But just that when a card have been inserted in the library then it will keep that id for the rest of the game. 
    1. There are quite a lot of text describing fixed ids, so you can relax a bit on that. 
1. Edit this both int the code, the roadmap and viosion document: If the full hand are cards that cannot be paid: then it is a autoloss for the player. 
    1. There are future steps for handling enemies. 
1. CardEffects that grants tokens and are under a cap: The cap will function a bit like the "cost" of other cards, just reversed: so high cost low cap and the reverse. So the cap is a range min-max and then the tokens gained is a percentage of that value. 
    1. The token percentage is also a min-max amount. 
    1. This is how all of them should work. 
1. The "Milestone tokens" on combat is in plural, around 100 of them. 
1. All the CardEffects that ChangeTokens should define the duration of the token: even each TokenType always have the same duration. 
    1. This is because in the future I would likely expand the duration possabilities of different TokenTypes.

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
