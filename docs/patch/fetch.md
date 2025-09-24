# Fetch a Patch

> `n34 patch fetch` command

**Usage:**
```
Fetches a patch by its id

Usage: n34 patch fetch [OPTIONS] <PATCH_ID>

Arguments:
  <PATCH_ID>  The patch id to fetch it

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
  -o, --output <PATH>              Directory for saving patches. Defaults to the current directory. Use `-` for stdout output
```

Fetches patches using their original patch ID. All fetched patches will be saved
to the specified output directory (current directory by default). You can then
apply or merge these patches into your branch as needed.

You can also write patches to stdout for direct application with tools like
`git-am`. For instance, to fetch and apply patches use:

```bash
n34 patch fetch -o '-' <note1...> | git am --empty=drop
```

The `--empty=drop` option ensures empty commits, such as cover letters, are
omitted.
