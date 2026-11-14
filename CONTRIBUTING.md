# Contributing

## Ways to contribute

You can experiment in your own copy for as long as you like. If you want to
share something, choose the route that fits.

- **Suggest an idea.** Start a local draft using the [idea template](.github/ISSUE_TEMPLATE/enhancement.md). Describe what you want to see or do on the radio and how you would tell it worked. For example, “On my X20S, I want to see receiver voltage from across the bench. When the receiver is off, I'd expect a dash. I don't know my ETHOS version.”
- **Try a widget and report back.** Fill in the [radio test report](.github/ISSUE_TEMPLATE/radio_test.md) with the steps you tried and what you saw. Both a successful check and a surprise help. If you share it, paste the report on the issue or pull request where the test was requested. For a standalone observation, choose **Radio test** under **Issues → New issue**.
- **Share an implementation.** Start with the [code contribution workflow](#workflow). A pull request (PR) lets a maintainer review a change from your own copy of the project.

An assistant can help with any route. Ask “Draft this for me here so I can
review it.” The draft stays local. You decide whether to paste the idea into an
**Idea or enhancement** issue, submit a test report, or share your code.

## Workflow

1. Open this repository in your assistant. Ask it to read [AGENTS.md](AGENTS.md)
   and find the open issue for the change. If a new issue is needed, review a
   local draft and submit it when ready. If you started from the parent
   `EthosLua` workspace, read its `AGENTS.md` and enter `sloppy-ethos/`.
2. Ask the assistant to prepare the [issue branch](AGENTS.md#branching-conventions)
   and run the [startup checks](AGENTS.md#required-startup-workflow). A branch
   keeps this contribution together while you try it.
3. Keep the change focused. Ask the assistant to follow the project's
   [versioning policy](docs/DEVELOPMENT.md#main-branch-release-and-versioning-policy)
   and [validation matrix](AGENTS.md#validation-matrix) for the files it changes.
   For installable script files or optional local assets, follow the
   [packaging documentation](docs/DEVELOPMENT.md#packaging-behavior).
4. Ask the assistant to draft the [PR description](.github/PULL_REQUEST_TEMPLATE.md)
   locally. Review the change, steps another pilot can repeat, what you observed,
   and what still needs a simulator or radio test.
5. When you choose to share, use **Fork** on GitHub to create a copy under your
   account. Have the assistant help you save the changes in a commit and
   upload (push) the contribution branch to **your fork**. On the original
   repository, choose **New pull request → compare across forks**, select the
   original project's `main` as the base and your fork's branch as the head.
   Check the displayed changes, paste your reviewed description, and submit it.
6. Link the issue. Use a closing keyword such as `Closes #29` when the PR
   resolves it, or `Refs #29` for partial work. Maintainers handle review and
   [merge](AGENTS.md#pr-merge-strategy), then follow the
   [release workflow](docs/DEVELOPMENT.md#release-workflow) for publication.

## Reference Checkouts

- Sibling directories in the parent `EthosLua` workspace are references, not
  part of this repo.
- Update reference repos only with commands scoped to that checkout, for
  example `git -C ../ETHOS-Feedback-Community pull --ff-only`.
- Treat reference projects as read-only evidence unless an issue or user request
  explicitly calls for updating them.
- Verify license and attribution before copying reference code, docs, or assets
  into `sloppy-ethos`.

## Documentation Changes

- Follow the [validation matrix](AGENTS.md#validation-matrix); documentation
  changes are complete when the relevant checks pass. Include the results in
  the PR `Verification` checklist.
- Follow [release guardrails](AGENTS.md#release-and-versioning-guardrails) for
  published download links; do not advertise unreleased script artifacts.
- Put script-owned tests under `scripts/{ProjectName}/tests/`; keep generic
  repository tooling and documentation contracts under root `tests/`.
- Follow [documentation and memory sync](AGENTS.md#documentation-and-memory-sync)
  for durable findings or handoff notes.

## Bug Report Quality

Follow [bug issue hygiene](AGENTS.md#bug-issue-hygiene) for templates, labels,
runtime details, and stable screenshot links.

## Commit Guidance

- Use concise, imperative commit messages.
- Include context in the body when behavior changes.
- Avoid mixing refactors and feature changes in the same commit when possible.

## Coding Notes

- Keep Ethos callbacks defensive (`paint`, `wakeup`, etc.) to avoid runtime crashes from unexpected callback invocation contexts.
- Prefer incremental performance changes; simulator can hit instruction limits quickly.
- Keep widget behavior deterministic and readable first, then optimize.
