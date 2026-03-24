When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Continue the work on the current branch, do not start a new branch. 
1. Implement the suggestions from: docs/suggestions-vision-roadmap.md
    1. About: "Clarify cost system semantics"-section: 
        1. Be sure that all "concrete cards" never have a percentage cost of anything. 
        1. That it is solely "card effects" that have percentage costs, but that when they are "rolled" to "concrete" cards, then the number is also concrete. 
        1. If this is not the case, then you are allowed to change the rust code for this sole purpose. 
            1. Do this before redooing the balance work.
    1. When all points are implemented then delete the file docs/suggestions-vision-roadmap.md
1. Implement most of the suggestions from /home/robbingdahood/.copilot/session-state/743a0019-5baf-46a7-ad54-bf7cf513ca90/research/the-current-session-for-optimizations-how-could-th.md 
    1. The "include_str!() config embedding"-idea should be clearly marked for runtime tests. It should not be usable on a normal build of the project. 
        1. Add test to ensure you cannot do anything with this new setup that you cannot do with a normal server. 
    1. Make sure to describe the "Parallel agents in worktrees — 3 background agents explore config variants simultaneously"-idea as a copilot skill in this repo, that can be used for the future. 
        1. Also use this new skill later in this plan when you start rebalancing again. 
        1. Add anything else from the ressearch to skills that can be used for that. 
        1. It is important that I have a powerfull setup that can give results faster through paralellism. 
1. The initial hand should not have been increased. 
    1. Instead increase the gain from the ressource card that draws cards. 
1. Shield should not be such a dominant mechanic.
    1. Reduce the number of shield cards. 
    1. Increase the benefit of the dodge. 
    1. I like the idea timely use of good dodges are more efficient than the "guranteed" use of a shield. 
1. I like that health and stamina starts at a 1000 
    1. So reset both back to 1000. 
    1. Then adjust the cards and card effects to balance the game. 
1. Redo the balancing checks again and keep adjusting the config files to achieve the balance as previous defined. 

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
