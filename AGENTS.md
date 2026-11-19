# AGENTS.md

Instructions for AI agents and contributors working on this repository.

## Project

zd-ccv is a cross-platform desktop viewer for SillyTavern character cards,
written in Rust with egui/eframe. See docs/PLAN.md for the full specification.

## Source of truth

docs/PLAN.md is the authoritative specification of the application.

- Read it before making changes to features or behavior.
- If an implementation decision deviates from or extends the plan, update
  PLAN.md in the same commit as the change.
- If code and PLAN.md disagree, PLAN.md wins unless the user explicitly
  decided otherwise.

## Documentation upkeep

These files must always reflect the current state of the application:

- docs/PLAN.md - specification: formats, parsing rules, UI layout, behavior.
- README.md - build, install, and run instructions plus user documentation
  (CLI usage, dialogs, panels, themes). Update it whenever a user-visible
  behavior, dependency, or command changes.
- AGENTS.md - this file. Update it when workflow, toolchain, or project
  structure changes.

## Workflow after every update

After each completed, working change (feature, fix, or doc update):

1. Run `cargo fmt` and `cargo test` and make sure they pass.
2. Run `cargo clippy` and fix warnings introduced by the change.
3. Increment the `build` field in `src/version.rs` by exactly one -
   never touch `major` or `minor` unless the user instructs it. The
   increment belongs to the same commit as the change, so HEAD's
   version always matches the binaries built below.
4. Commit with a concise message describing the change.
5. Push to the tracked remote branch (`git push`).
6. Build the binaries so the local ones match the pushed commit -
   `cargo build --release` (and `cargo build` for debug). The user tests
   against these binaries, so never leave them behind HEAD.

Commit small, commit often: one logical change per commit. Never commit a
state where `cargo test` fails. Never commit secrets.

## Conventions

- Rust 2021 edition, stable toolchain.
- Keep the card parser (src/card.rs) free of GUI dependencies so it stays
  unit-testable against samples/.
- samples/ is the fixture set for parser tests (gitignored for size; do not
  weaken tests to accommodate its absence - skip sample-driven tests cleanly
  when samples/ is missing).
- Do not add new dependencies without a clear need; prefer the existing set:
  eframe/egui, clap, rfd, image, serde_json, base64, flate2, dark-light.
