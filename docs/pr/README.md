# Pull Request Management

Pull requests are a new addition to [NIP-34], proposed by [DanConwayDev] in
[nostr-protocol/nips#1966]. The concept is straightforward: it involves an event
that notifies you of changes on a Git server, which can be any Git server. This
event contains two key pieces of information, the Git repository URL and the
commit tip. You can add this remote to your project, check out the specified
commit, and then review or merge/rebase the changes as needed. Additionally,
there is a `PR-update` event that informs you when the previous commit tip is no
longer the head of the changes, such as when new changes are added or a rebase
occurs.

## Why Use the Commit Tip?

Using the commit tip is preferable to relying on a branch name. A branch name
simply points to a commit, so checking out a branch is effectively the same
as checking out a specific commit. However, if we use the branch name instead
of the commit, the changes can be altered by anyone, not just the pull request
author. By checking out a specific commit ID, we ensure that the changes are
exactly as intended by the pull request author and have not been tampered with
by a malicious party.

## Does `n34` Handle Git Operations?

No, `n34` does not perform any Git operations. You can use the `n34 pr view`
command to retrieve the pull request clone URL and the commit tip, allowing
you to fetch the changes manually. Similarly, `n34 pr new` and `n34 pr update`
do not interact with Git directly. They only prompt you for the clone URL and
commit tip to construct the event. For more details on the philosophy behind
`n34`, refer to the [Philosophy] section.

## Why use pull requests instead of patches?

Pull requests are useful for large changes. Most relays limit events to 60KB,
so if your changes exceed that limit (per event, not in total), consider using
pull requests. For [GRASP] servers, the limit is usually higher. Before using pull
requests, check the relays, patches are more decentralized.

[NIP-34]: https://github.com/nostr-protocol/nips/blob/master/34.md
[DanConwayDev]: https://danconwaydev.com
[nostr-protocol/nips#1966]: https://github.com/nostr-protocol/nips/pull/1966
[Philosophy]: https://n34.dev/commands.html#philosophy
[GRASP]: https://ngit.dev/grasp
