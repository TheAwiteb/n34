# Reopen a Patch

> `n34 patch reopen` command

**Usage:**
```
Reopens a closed or drafted patch

Usage: n34 patch reopen [OPTIONS] <PATCH_ID>

Arguments:
  <PATCH_ID>  The closed/drafted patch id to reopen it. Must be orignal root patch

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
```

Issue a kind `1630` (Open status) for the specified patch. The patch have to
be closed or drafted.

