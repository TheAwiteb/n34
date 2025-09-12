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
  -o, --output <PATH>              Output directory for the patches. Default to the current directory
```

Fetches patches using their original patch ID. All fetched patches will be saved
to the specified output directory (current directory by default). You can then
apply or merge these patches into your branch as needed.
