# Contributing to Kamishibai

Thanks for your interest! The project is very young, so the best way to help
right now is to open an issue with ideas, questions or bugs.

## Before opening a pull request

Run the same checks as CI:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Guidelines

- Every change a user (or an AI) can make to a project should go through an
  `EditCommand`, so it can be undone.
- Keep code that touches Bevy APIs small and well isolated: Bevy is pre-1.0
  and we upgrade with each release.
- Add a test for logic that doesn't need a window or a GPU.

By contributing, you agree that your work is dual licensed under MIT and
Apache-2.0, as described in the README.
