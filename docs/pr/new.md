# Create a Pull Request

> `n34 pr new` command

**Usage:**
```
Usage: n34 pr new [OPTIONS] <--subject <SUBJECT>|--editor> <COMMIT> <CLONES>...

Arguments:
  <COMMIT>     The SHA-1 hash of the commit at the tip of the PR branch
  <CLONES>...  Repositories to clone for the pull request, separated by commas

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
      --body <BODY>                The body content of the pull request. Cannot be used together with the `--editor` flag
      --subject <SUBJECT>          The subject or title of the pull request. Cannot be used together with the `--editor` flag
  -e, --editor                     Opens the user's default editor to write PR subject and body
      --labels <LABELS>            Labels to associate with the pull request, separated by commas
      --branch <BRANCH>            The branch name for the pull request
      --grasp                      Push the pull request to the repository GRASP server
```

Submit a pull request to the repositories specified using the `--repo` option or
obtained from the `nostr-address` file.

Utilize the `--grasp` option if you intend to send the pull request to the
[GRASP] servers. Note that `n34` will not automatically send the pull request
for you but will indicate where to push your changes.

[GRASP]: https://ngit.dev/grasp
