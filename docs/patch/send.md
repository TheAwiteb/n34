# Send Patches to a Repository

> `n34 patch send` command

**Usage:**
```
Send one or more patches to a repository

Usage: n34 patch send [OPTIONS] <PATCH-PATH>...

Arguments:
  <PATCH-PATH>...  Space-separated list of patch files to send. Use `-` to read from stdin.

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
      --original-patch <EVENT-ID>  Original patch ID if this is a revision of it
```

Send your generated patches to the repositories specified using the `--repo`
option or retrieved from the `nostr-address` file. When submitting a revision
of an existing patch, include the original patch ID to ensure it’s correctly
referenced in your revision patch event.

You can also pass patches from stdin, retrieved from a website or directly from
`git-format-patch`. Simply use `-` as the patch path. For example:

```bash
git format-patch --stdout --base master master..HEAD | n34 patch send -
```
