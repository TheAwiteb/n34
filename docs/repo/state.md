# Repository State Announcements

> `n34 repo state` command

**Usage:**
```
Repository state announcements

Usage: n34 repo state [OPTIONS] <HEAD>

Arguments:
  <HEAD>  Name of the repository's primary branch, such as 'master' or 'main'

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository addresses
      --tags <TAGS>                Tags to announce a state for, in the format `<tag-name>=<commit-id>`. Separated by comma
      --branches <BRANCHES>        Branches to announce a state for, in the format `<branch-name>=<commit-id>`. Separated by comma
```

This command allows you to announce your repository state, which is useful for
pushing to permissionless git repositories like the [GRASP] relay. The relay
will verify your repository state and permit pushing commits only if they match
the announced state.

To get the commit ID that a branch or tag points to, use `git rev-parse
<tag-or-branch-name>`. You can automate this process by creating a script to
generate the required input for this command.

[GRASP]: https://ngit.dev/grasp
