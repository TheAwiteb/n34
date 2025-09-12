# Close a Patch

> `n34 patch close` command

**Usage:**
```
Closes an open or drafted patch

Usage: n34 patch close [OPTIONS] <PATCH_ID>

Arguments:
  <PATCH_ID>  The open/drafted patch id to close it. Must be orignal root patch

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
```

Issue a kind `1632` (Close status) for the specified patch. The patch have to
be open or drafted.

