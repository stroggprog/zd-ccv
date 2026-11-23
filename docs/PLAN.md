# zd-ccv

zd-ccv is a cross-platform desktop application that allows the user to view
the contents of one or more character cards: SillyTavern cards (V1/V2/V3)
and AICC cards ("Rin Cards", the Rin Chat format), both stored as JSON
inside PNG files.

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
- `chara`/`ccv3` - AICC card ("Rin Cards", `spec: aicc_card`,
  `spec_version: 1.0`): Rin Chat embeds its native JSON under the same
  SillyTavern keywords, so the format is detected from the `spec` tag.

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
 6. AICC cards (`spec: "aicc_card"`) are labeled as "AICC Card" and
    rendered from their native nested structure (see below).

## Input handling

- Only PNG files are loaded. The application accepts any number of files
  and/or folders on the command line; a file is considered only if its
  extension is `.png` (case-insensitive), and folders are scanned
  recursively for `.png` files. Files with any other extension (`.webp`,
  `.json`, etc.) are ignored, whether passed explicitly or found in a
  folder.
- With no arguments, the application opens a native folder picker.
- The menu bar provides File > Open Files... (multi-select, filtered to
  PNG), File > Open Folder..., File > About, and File > Quit. Dialog selections replace
  the current list.
- A card PNG without any recognized chunk shows the image plus a warning
  ("image contains no character card data" - the file was likely
  re-saved or compressed by an outside tool).

## UI layout

- Top: a simple menu bar with the option File (Open Files..., Open
  Folder..., About, Quit). About opens a modal dialog (centered, dimmed
  backdrop) showing the application name, the version in
  `major.minor.build` format, a one-line description, a hyperlink to the
  GitHub repository that opens the system web browser
  (`https://github.com/stroggprog/zd-ccv`), a "Check 4 Update" button,
  and a Close button; it also closes on Escape or a click outside the
  dialog.
- Below the menu bar, a toolbar:
  - `<` and `>` buttons step to the previous/next card in the loaded list
    (disabled at the ends; wrap-around is not used).
  - A label "n / N - filename" shows position and current file.
  - A theme selector: System / Light / Dark.
- Directly below the toolbar, a second fixed row carries the export
  buttons: "Export Character", "Export Lorebook" (disabled when the
  displayed chunk embeds no lorebook), and "Export All"; the whole row
  is disabled when no card data is displayed. Keeping them on their
  own row gives the buttons a constant position no matter how long the
  current file name is.
- When an export has been performed (or failed), a status bar strip at
  the bottom of the window shows the result until dismissed, replaced,
  or the card changes.
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
    - SillyTavern blocks appear in spec order: name, nickname (V3), description,
      personality, scenario, first_mes, alternate greetings, mes_example
      (rendered per `<START>` dialogue block), creator_notes,
      creator_notes_multilingual (V3), system_prompt,
      post_history_instructions, tags, creator, character_version,
      source (V3), creation/modification dates (V3), embedded lorebook
      (`character_book`, entry count + per-entry blocks), extensions
      (as formatted JSON), then the raw JSON of the selected chunk in a
      collapsible block.
    - AICC blocks appear in the format's own export order: Name,
      Description (`general_description`), Appearance, Personality
      (`personality.core`), Behavior Rules, Speech Tone/Verbosity/Format/
      Patterns, Background & History, World, Setting & Context, Greetings,
      Example Dialogue, Group-Only Greetings, System Prompt, Post-History
      Instructions, Depth Prompt (`prompts.depth_prompt.text`), Worldbook
      (rendered like a lorebook from `world.worldbook_entries`), Display
      Name, Creator, Creator Notes, Card Version, Tags, Card Format; plus
      JSON blocks for optional extras (Generators, Regex Scripts,
      Features, Assets, Doc Styles, Chat Style) when present.
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

## Export

All exports serialize the currently displayed chunk (`ccv3`/`chara`
selection in the data panel) as pretty-printed JSON, written next to the
source image (no save dialog); existing files are overwritten, and the
status bar reports the written path:

- Export Character: the card object with any embedded lorebook
  (`data.character_book`) removed, to the source file's name with a
  `.json` extension.
A `"character_book"` that is absent or `null` - common in real cards -
  counts as no lorebook for both the button state and the export.
- Export Lorebook: the embedded `character_book` object as a standalone
  lorebook JSON, to a file named after the lorebook's own `name` field
  (sanitized for the filesystem) with a `.json` extension, in the
  source file's folder. Disabled when the displayed chunk has no
  lorebook. If the lorebook has no completed `name`, the character's
  name is inserted into the exported `name` field and the file is named
  after the character with `_lore` appended (e.g. character "Dia" with
  an unnamed lorebook exports `Dia_lore.json` with `"name": "Dia"`).
- Export All: the complete embedded payload, including the lorebook, to
  the source file's name with a `.json` extension (this is the standard
  SillyTavern card JSON export shape).
- AICC cards: Export Lorebook maps `world.worldbook_entries` (plus
  `worldbook_name`) into the standalone lorebook shape; the button is
  enabled only when that book has at least one entry. The lorebook's
  name-file fallback applies as usual (`<worldbook name>.json`, else
  `<character name>_lore.json`). Export Character keeps `world` intact -
  it is part of the AICC card object, not a separable book.

## Versioning

- The version lives in `src/version.rs` as a `Version` struct with three
  fields: `major`, `minor`, and `build`. It is the single source of
  truth for both the About dialog and `zd-ccv --version`.
- `major` and `minor` change only when the user instructs it.
- `build` increments by one every time the project is built, as part of
  the same commit so HEAD always matches the shipped binary's version.
- The `version` field in `Cargo.toml` mirrors the struct; a unit test
  (`cargo_manifest_version_stays_in_sync`) fails if the two ever drift.
- The About dialog's "Check 4 Update" button fetches the repository's
  `Cargo.toml` from the `main` branch via
  `raw.githubusercontent.com`, reads the `[package]` `version` (only
  that section counts; dependency sections may have their own `version`
  keys), and compares `major.minor.build` with the local version. Any
  remote triple greater than the local one is treated as a newer
  release.
- The fetch runs on a background thread (ureq with rustls, 10 s global
  timeout, 64 KiB read limit) so the UI never blocks; while running the
  button shows a spinner and a re-check is ignored. The outcome is
  shown in the About dialog and mirrored to the status bar so it
  survives closing the dialog. Failures (network, HTTP status,
  unparseable manifest) report "Update check failed: <reason>." in
  warning color.

## Out of scope (v1)

- Editing or re-writing card PNG files (the JSON exports are one-way;
  exported `.json` files are not re-imported).
- CHARX containers and V3 `chara-ext-asset_:` embedded assets.
- Rendering of `extensions` sub-fields beyond raw JSON.
