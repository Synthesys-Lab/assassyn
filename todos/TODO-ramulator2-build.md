# Goal

1. Make the build script for ramulator2, `../scripts/init/ramulator2.sh`, idempotent.
2. Update the existing patch `../scripts/ramulator2-template.patch` as per the updated document `../scripts/ramulator2-patch.md`.

# Action Items

1. The existing `../scripts/init/ramulator2.sh` is not idempotent because it uses `git apply` to apply a patch, which fails if the patch has already been applied. Modify the script to check if the patch has already been applied before attempting to apply it. If it has been applied, skip the patching step. Alternatively, reset any changes in the `3rd-party/ramulator2` directory before applying the patch to ensure a clean state.
  - This is for you to decide.
  - After making the changes, run `source ../scripts/init/ramulator2.sh` twice in `zsh` to make sure it works.
    - Commit here without verification.
2. Review the document `../scripts/ramulator2-patch.md` to understand the changes that need to be made to the existing patch `../scripts/ramulator2-template.patch`.
  - The write back hook is a TODO in `ramulator2/dram_controller/impl/generic_dram_controller.cpp`, find that and modify it manually.
  - Make sure it compiles.
  - Pack the changes into a new patch file that overwrites the old `../scripts/ramulator2-template.patch`, commit here without verification!
  - Make sure the new patch can be applied cleanly to a fresh clone of the ramulator2 repository.
3. Add the idempotent ramulator2 build script to the beginning of `scripts/pre-commit` to ensure it checks before every commit.
  - Commit here with verification!

# Checklist

Summarize your changes here.
