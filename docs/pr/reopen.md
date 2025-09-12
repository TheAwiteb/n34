# Reopen a Pull Request

> `n34 pr reopen` command

**Usage:**
```
Usage: n34 pr reopen [OPTIONS] <PR_ID>

Arguments:
  <PR_ID>  The closed/drafted patch id to reopen it

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
```

Issue a kind `1630` (Open status) for the specified PR. The PR have to
be closed or drafted.
