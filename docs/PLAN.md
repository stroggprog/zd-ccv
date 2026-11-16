# zd-ccv

zd-ccv is a cross-platform desktop application that allows the user to view
the contents of one or more SillyTavern character cards.

## Technology

Rust with egui/eframe. Deliberate choices:

- `eframe`/`egui` for the GUI (pure Rust, built-in menu bar and light/dark
  visuals, persisted application state).
- `clap` for CLI argument parsing, `rfd` for native file dialogs,
  `image` for portrait decoding, `dark-light` for OS theme detection,
  `serde_json`/`base64`/`flate2` for card data extraction.
- The card parser (`src/card.rs`) has no GUI dependencies and is unit
  tested against `samples/`.

## Card format

SillyTavern cards are ordinary PNG images whose character data is a JSON
document, UTF-8 encoded and base64-encoded, stored in PNG `tEXt` metadata
chunks (not in the file header). The chunks appear after the `IDAT` data,
immediately before `IEND`.

Chunk keyword and spec version:

- `chara` - Character Card V1/V2 (`spec: chara_card_v2`).
- `ccv3`  - Character Card V3 (`spec: chara_card_v3`), a superset of V2.

Parsing rules:

1. Verify the PNG signature, then walk every chunk in the file. Collect the
   payloads of all `tEXt`, `iTXt`, and `zTXt` chunks whose keyword
   (case-insensitive) is `chara` or `ccv3`.
2. Base64-decode each payload and parse it as JSON. A chunk that fails
   either step is reported as a warning block; the image is still shown.
 3. When both `ccv3` and `chara` chunks are present, `ccv3` takes precedence
    (per the Character Card V3 spec). The displayed default follows this
    order: `ccv3` first, then `chara`.
 4. Duplicate chunks of the same keyword are common (real-world files carry
    two `chara` chunks) and may legitimately hold different data, since
    SillyTavern reads only the *first* `chara` chunk while other tools may
    append a second one. The viewer keeps every chunk, defaults to the one
    SillyTavern would use (the first of the highest-precedence keyword), and
    offers a dropdown so the user can select which chunk to display. A chunk
    that fails to decode is skipped with a warning; the rest remain
    selectable.
 5. V1 cards are flat JSON without a `spec` wrapper; they are normalized to
    the V2 field model and labeled as V1.

## Input handling

- Only PNG files are loaded. The application accepts any number of files
  and/or folders on the command line; a file is considered only if its
  extension is `.png` (case-insensitive), and folders are scanned
  recursively for `.png` files. Files with any other extension (`.webp`,
  `.json`, etc.) are ignored, whether passed explicitly or found in a
  folder.
- With no arguments, the application opens a native folder picker.
- The menu bar provides File > Open Files... (multi-select, filtered to
  PNG), File > Open Folder..., and File > Quit. Dialog selections replace
  the current list.
- A card PNG without any recognized chunk shows the image plus a warning
  ("image contains no character card data" - the file was likely
  re-saved or compressed by an outside tool).

## UI layout

- Top: a simple menu bar with the option File (Open Files..., Open
  Folder..., Quit).
- Below the menu bar, a toolbar:
  - `<` and `>` buttons step to the previous/next card in the loaded list
    (disabled at the ends; wrap-around is not used).
  - A label "n / N - filename" shows position and current file.
  - A theme selector: System / Light / Dark.
- Two main panels side by side:
  - The left panel displays the portrait, scaled to fit the panel; the
    mouse wheel zooms; the panel scrolls when zoomed beyond fit.
  - The right panel displays the card data and is scrollable. Each field is
    in a separate visually delimited block (header + content). Its scroll
    position is pinned to the top for a short window after the displayed
    card changes or a different embedded chunk is selected - the panel is
    re-pinned each frame until egui's residual wheel smoothing has decayed
    (bounded frame budget), so a tall description cannot end up displayed
    halfway down right after a switch.
    - When the file carries more than one embedded chunk, an "Embedded
      chunk:" dropdown at the top of the panel lists every decoded chunk
      (e.g. "chara 1 of 2 (Character Card V2)"); selecting one switches the
      fields and raw JSON below to that chunk. The default selection is the
      chunk SillyTavern would use.
    - Blocks appear in spec order: name, nickname (V3), description,
      personality, scenario, first_mes, alternate greetings, mes_example
      (rendered per `<START>` dialogue block), creator_notes,
      creator_notes_multilingual (V3), system_prompt,
      post_history_instructions, tags, creator, character_version,
      source (V3), creation/modification dates (V3), embedded lorebook
      (`character_book`, entry count + per-entry blocks), extensions
      (as formatted JSON), then the raw JSON of the selected chunk in a
      collapsible block.
    - Empty or absent fields render as a muted "not set" block so the
      order stays stable across cards.
    - Warning blocks (undecodable chunks, missing data) render first,
      styled distinctly.

## Themes and persistence

- Light and dark themes are selectable by the user; the choice is
  persisted and restored on next start.
- On first startup the effective theme follows the desktop theme
  (System mode, detected via the `dark-light` crate; re-checked per run).
- Application state (the theme choice) is persisted through eframe's
  built-in storage; no separate config file is written.

## Out of scope (v1)

- Editing or writing card files of any kind.
- CHARX containers and V3 `chara-ext-asset_:` embedded assets.
- Rendering of `extensions` sub-fields beyond raw JSON.
