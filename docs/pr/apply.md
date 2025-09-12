# Mark as Applied

> `n34 pr apply` command

**Usage:**
```
Usage: n34 pr apply [OPTIONS] <PR_ID> <APPLIED_COMMITS>...

Arguments:
  <PR_ID>               The open PR id to apply it
  <APPLIED_COMMITS>...  The applied commits

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
```

Creates a kind `1631` event (Applied/Merged status) for the specified PR. The PR
must be in open status.

The `APPLIED_COMMITS` field serves to inform clients about the status of
specific commits, whether they have been applied or not. If you need to retrieve
the list of commits from a specific point (such as the tip of the master branch)
up to the `HEAD`, you can use the following Git command: `git log --pretty=%H
'origin/master..HEAD'`.
