# List Patches

> `n34 patch list` command

**Usage:**
```
List the repositories patches

Usage: n34 patch list [OPTIONS] [NADDR-NIP05-OR-SET]...

Arguments:
  [NADDR-NIP05-OR-SET]...  Repository addresses

Options:
      --limit <LIMIT>  Maximum number of patches to list [default: 15]
```

List the repositories patches. By default `n34` will look for `nostr-address`
file and extract the repositories from it.

