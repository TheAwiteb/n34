# Convert to Draft

> `n34 patch draft` command

**Usage:**
```
Converts an open patch to draft state

Usage: n34 patch draft [OPTIONS] <PATCH_ID>

Arguments:
  <PATCH_ID>  The open patch id to draft it. Must be orignal root patch

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
```

Issue a kind `1633` (Draft status) for the specified patch. The patch have to
be open.
