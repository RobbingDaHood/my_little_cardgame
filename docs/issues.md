When the below point states "Roadmap" it means edit the roadmap.md directly.

1. Fix all the know errors stated in docs/design/known_failures.md 
    1. If in doubt how to correct the tests then ask me. 
    1. Also correct the instructions file in this repo to never accept failing tests before pushing code and ask me if in doubt about handling the automatic test.  
    1. Remove any mention about docs/design/known_failures.md file and delete the file afterwards. 
1. Roadmap, add to "Finalize edge cases for a repeatable loop and concurrency":
    1. There should not be any file persistance on the server. 
    1. "Save games" are based on the action log: Players can query the full length of that and store it locally together with the version of the game and the seed. 
        1. Also add to this step to expose a unique version code for each compiled game.
    1. Then with this actionlog, seed and version then the player can "load" any game again. 
    1. Ensure this is possible through our endpoints. 
        1. Estimate how big an action log could be come, when it would not be viable to load this through a "load game"-"player action" and if it makes sense to have a dedicated laod game post endpoint that takes a file as payment: does that make a difference? 
1. Is there any reason to keep "scripts/check_fmt.sh" when we have "scripts/check_all.sh"?        
1. Roadmap: IN "UX polish, documentation, tools for designers, and release": Make sure that there is exposed documentation at endpoints (maybe the openapi) on: 
    1. A tutorial for new players (that should be linked to from the README.md, just a link of a running server). 
    1. The openapi specification should give ideas about general mentality of each diciplin and action. More than just simple specifications. 
    1. Also have some "hints" page with good strategies etc. 
    1. The goal is that anyone could play the game solely with this documentation.
    1. Add to the instructions file to always keep this documentation up to date. 

# When done with all of this then update vision and roadmap files

If I instructed you to do something that you could not read from those two files (Except instructions above to edit the roadmap or vision files directly), then change those files so it is more clear. 

Also, make general improvement to both files.
