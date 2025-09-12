# Close a Pull Request

> `n34 pr close` command

**Usage:**
```
Usage: n34 pr close [OPTIONS] <PR_ID>

Arguments:
  <PR_ID>  The open/draft PR id to close it

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
```

Issue a kind `1632` (Close status) for the specified PR. The PR have to be open
or drafted.

