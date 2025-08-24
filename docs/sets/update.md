# Modify a Set

> `n34 sets update` command

**Usage:**
```
Modify an existing set

Usage: n34 sets update [OPTIONS] <NAME>

Arguments:
  <NAME>  Name of the set to update

Options:
      --set-relay <RELAYS>         Add relay to the set, either as URL or set name to extract its relays. [aliases: `--sr`]
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
      --override                   Replace existing relays/repositories instead of adding to them
```

Use this command to update an existing set by its name. By default, providing
relays via `--set-relay` or repositories via `--repo` will add them to the set's
existing entries. To replace the current relays and repositories with the new
values, use the `--override` flag.

[passing repositories]: /commands.html#passing-repositories
