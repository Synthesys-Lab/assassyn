# Fix Git Cache Hit for Unify Library Path

This is a follow up to [unifying lib path](../dones/DONE-unify-lib-path.md). Two lib path files `.cwrapper-lib-path` and  `.ramulator2-lib-path` are not part of [wrapper](../tools/c-ramulator2-wrapper/), which does not trigger a rebuild, which shall be solved. Otherwise, we do not have these two files at all to properly test.

## Action Items

1. Make `.cwrapper-lib-path` and `.ramulator2-lib-path` generated in wrapper build folder.
2. Refer to [unifying lib path](../dones/DONE-unify-lib-path.md) find all the files referring to these files use `$ASSASSYN_HOME/../tools/c-ramulator2-wrapper/build`
3. Currently, we have two separate scripts for [wrapper](../scripts/init/wrapper.sh) and [ramulator2](../scripts/init/ramulator2.sh). Merge these two into one, as wrapper always depends on this.
4. Fix the `pre-commit` hook that runs `ramulator2.sh` by running `wrapper.sh`.