# ZPU debugging log.

# Thu Jun  8 20:46:43 -00 2017
Basics OPs definitly work, they did earlier.

Now, the hardware EMULATE implementations seem to work as well.

Still needs another round of checking, but my test binary (reb) works.
It's relatively fast as well.

# Sat May 13 03:37:01 -00 2017
At this point, my REB zpu_phi binary runs more or less
sucessful until about 2635 instructions.

A NEQBRANCH gets taken even though it shouldn't, me thinks.
Might be another issue entirely, but it's the closest I have right now.

More investigation is definitly needed.

# Sun May 14 00:32:18 -00 2017
It works!

The non-EMULATE instructions work!

That along with serial I/O, the memory bus and who knows what else.

Big step. Next up, EMULATE instructions. Shouldn't take too long either.
