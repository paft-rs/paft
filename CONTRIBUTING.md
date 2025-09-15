# Contributing to paft

Thank you for your interest in contributing! We welcome issues, bug fixes, new features, and documentation improvements.

## Getting Started

- Ensure you have a recent Rust toolchain installed (`rustup` recommended).
- Clone the repository and run the test suite:
  
  ```bash
  cargo test --workspace --all-features
  ```

- Run linting locally to catch common issues:
  
  ```bash
  cargo clippy --workspace --all-features -- -W clippy::all -W clippy::cargo -W clippy::pedantic -W clippy::nursery -D warnings
  ```

You can also use the `justfile` helpers if you have `just` installed:

```bash
just test             # run all tests
just lint             # run clippy with strict warnings
```

## How to Contribute

1. Fork the repository and create your branch from `main`.
2. If you’ve added code that should be tested, add tests.
3. Ensure the test suite passes and lints are clean.
4. Update documentation, comments, and examples as needed.
5. Open a pull request with a clear title and description of your changes.

## Commit and PR Guidelines

- Keep commits focused and logically grouped.
- Use conventional-style commit messages when possible (e.g., `feat:`, `fix:`, `docs:`).
- Reference related issues in your PR description (e.g., `Closes #123`).

## Coding Standards

- Prefer clarity over cleverness.
- Add tests for new functionality and edge cases.
- Avoid unnecessary allocations; prefer borrowing where appropriate.
- Keep public APIs well-documented with `rustdoc`.

## Reporting Security Issues

If you discover a security vulnerability, please do not open a public issue. Report it privately to the maintainers at [heydays_micron_7m@icloud.com](mailto:heydays_micron_7m@icloud.com). We will respond promptly.

## Questions and Support

For general questions, discussions, or help getting started, please open a GitHub Discussion or issue. You may also reach the maintainers at [heydays_micron_7m@icloud.com](mailto:heydays_micron_7m@icloud.com).

We appreciate your contributions!
