# Merge a Pull Request

> `n34 pr merge` command

**Usage:**
```
Usage: n34 pr merge [OPTIONS] <PR_ID> <MERGE_COMMIT>

Arguments:
  <PR_ID>         The open PR id to merge it
  <MERGE_COMMIT>  The merge commit id

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
```

Creates a kind `1631` event (Applied/Merged status) for the specified PR. The
PR must be in open status.

You can get the `MERGE_COMMIT` commit using `git rev-parse HEAD` command if
the merge commit in the `HEAD` or use `HEAD~n` where the `n` is the number of
commits the merge commit before the HEAD
