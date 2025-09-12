# Update a Pull Request

> `n34 pr update` command

**Usage:**
```
Usage: n34 pr update [OPTIONS] <EVENT-ID> <COMMIT> <CLONES>...

Arguments:
  <EVENT-ID>   Original PR ID
  <COMMIT>     The SHA-1 hash of the commit at the tip of the PR branch
  <CLONES>...  Repositories to clone for the pull request, separated by commas

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
      --grasp                      Push the pull request update to the repository GRASP server
```

Update an existing pull request with the new changes.

Utilize the `--grasp` option if you intend to send the pull request to the
[GRASP] servers. Note that `n34` will not automatically send the pull request
for you but will indicate where to push your changes.

[GRASP]: https://ngit.dev/grasp
