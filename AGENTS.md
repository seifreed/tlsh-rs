# Repository Guidelines

## Project Structure & Module Organization
`src/lib.rs` is the public library entry point. Core TLSH logic lives in `src/builder.rs`, `src/digest.rs`, `src/profile.rs`, `src/error.rs`, and `src/internal/` for lower-level helpers and constants. The CLI is split by concern under `src/cli/`, with the executable wrapper in `src/bin/tlsh.rs`. Integration coverage lives in `tests/`: `tests/cli.rs` exercises the command-line surface, and `tests/compat.rs` validates known TLSH vectors and profile compatibility. Keep reusable sample inputs in `fixtures/`.

## Build, Test, and Development Commands
Use stable Rust 1.85+ with edition 2024.

- `cargo build --locked` builds the library and CLI exactly as CI does.
- `cargo build --release --bin tlsh` produces the optimized `tlsh` binary.
- `cargo test --locked --all-targets` runs unit, integration, and binary tests.
- `cargo fmt --check` verifies formatting before pushing.
- `cargo run --bin tlsh -- hash ./fixtures/small.txt` is the fastest local CLI smoke test.

## Coding Style & Naming Conventions
Follow `rustfmt` defaults; use `cargo fmt` before opening a PR. Match the existing Rust style: 4-space indentation, `snake_case` for functions/modules/tests, `CamelCase` for types, and concise enums/struct names that map to TLSH concepts. Prefer small modules with focused responsibilities, and add new CLI behavior inside the existing `args` / `application` / `presentation` split rather than growing `main`.

## Testing Guidelines
Add unit tests next to implementation under `#[cfg(test)]` when behavior is local to one module. Add integration tests in `tests/` when validating public APIs, CLI output, or compatibility vectors. Name tests for the observable behavior, for example `parse_rejects_unknown_command` or `cli_xref_supports_json_output`. Preserve the current standard of high coverage by adding regression tests for every bug fix and new profile/format branch.

## Commit & Pull Request Guidelines
The current history uses short, imperative subjects such as `Remove project plan and gitignore from repository`. Keep commit titles concise, imperative, and specific. For pull requests, include:

- A brief summary of the behavioral change.
- Linked issue or rationale when no issue exists.
- Test evidence, usually `cargo fmt --check` and `cargo test --locked --all-targets`.
- Example CLI output when changing JSON, SARIF, or text formatting.
