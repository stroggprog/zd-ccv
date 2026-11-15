# zd-ccv

A cross-platform desktop viewer for SillyTavern character cards.

Character cards are PNG images that carry the character definition as
base64-encoded JSON inside PNG `tEXt` metadata chunks (`chara` for
V1/V2 cards, `ccv3` for V3). zd-ccv shows the portrait next to every
data field the card contains, read-only.

## Features

- View one or many cards: files and/or folders on the command line, or
  pick them through native dialogs.
- Supports Character Card V1, V2, and V3 data embedded in PNG files, and
  plain `.json` card exports.
- Portrait viewer with fit-to-panel scaling and mouse-wheel zoom.
- Scrollable data panel: one delimited block per field, in spec order,
  including embedded lorebooks and raw JSON of each embedded payload.
- Light and dark themes; first start follows the desktop theme, your
  choice is remembered.
- Diagnostics for broken cards: missing metadata, undecodable chunks,
  conflicting duplicate chunks.

## Requirements

- Rust (stable toolchain) - https://rustup.rs
- Linux additionally needs GTK 3 and WebKitGTK development packages for
  the native file dialogs:
  - Debian/Ubuntu: `libgtk-3-dev libwebkit2gtk-4.1-dev`
  - Fedora: `gtk3-devel webkit2gtk5.0-devel` (or `webkit2gtk4.1-devel`)
- Other typical desktop libraries per `winit`/`glutin` requirements
  (`libxcb`/`libxkbcommon`/`mesa` on X11, or a Wayland session).

## Build

```sh
cargo build --release
```

The binary ends up in `target/release/zd-ccv`.

## Run

```sh
# Pick a folder through a dialog
zd-ccv

# One or more card files
zd-ccv alice.png bob.png

# A folder (scanned recursively for .png/.webp/.json files)
zd-ccv ~/SillyTavern/data/default-user/characters

# A mixed list
zd-ccv somecards/ featured.json
```

## Install

Copy the release binary anywhere on your `PATH`, e.g.:

```sh
cargo build --release
install -m 755 target/release/zd-ccv ~/.local/bin/
```

## Using the application

- **Menu bar** - File > Open Files... picks one or more files,
  File > Open Folder... picks a directory. Both replace the current
  list. File > Quit exits.
- **Toolbar** - `<` and `>` step through the loaded cards; the label
  between them shows position and file name. On the right side the
  theme selector switches between System, Light, and Dark.
- **Left panel** - the card portrait, scaled to fit. Scroll the mouse
  wheel over it to zoom; scroll to pan when zoomed.
- **Right panel** - all card fields, each in its own block: name,
  description, personality, scenario, first message, alternate
  greetings, example messages, creator notes, prompts, tags, lorebook
  entries, and the raw JSON of each embedded payload (collapsible).
  Unset fields appear as muted "not set" blocks. Warning blocks are
  shown above the fields when a file has no card data or contains
  conflicting metadata.

## Development

```sh
cargo test        # parser test suite (uses samples/ fixtures if present)
cargo clippy
cargo fmt
```

The card parser in `src/card.rs` is GUI-free and unit-tested. The
`samples/` directory is a local, gitignored fixture set; sample-driven
tests skip cleanly when it is absent.

The specification lives in [docs/PLAN.md](docs/PLAN.md) and is the
source of truth; README and PLAN are kept up to date with every change.
