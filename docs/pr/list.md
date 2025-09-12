# List Pull Requests

> `n34 pr list` command

**Usage:**
```
Usage: n34 pr list [OPTIONS] [NADDR-NIP05-OR-SET]...

Arguments:
  [NADDR-NIP05-OR-SET]...  Repository addresses

Options:
      --limit <LIMIT>              Maximum number of patches to list [default: 15]
```

List the repositories pull requests. By default `n34` will look for
`nostr-address` file and extract the repositories from it.
