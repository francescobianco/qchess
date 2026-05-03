---
name: qchess project overview
description: TUI chess app in Rust — architecture, stack, design decisions
type: project
originSessionId: b53f3c15-ba19-48ea-8891-2c31ff392650
---
Rust TUI chess database browser. Any folder with .pgn files is a valid database.

**Stack**: ratatui 0.29, crossterm 0.28, shakmaty 0.28, pgn-reader 0.27, walkdir 2, image 0.24 (0.25 richiede Rust 1.88, installato 1.86), base64 0.22

**Board rendering**: PNG via backend auto Kitty/SIXEL. Kitty viene usato su kitty/WezTerm; SIXEL viene scelto su VTE/GNOME Terminal quando rilevabile, cosi' la scacchiera PNG funziona anche su Ubuntu GNOME. Fallback Unicode automatico con `--unicode`, `--graphics unicode`, o se non viene rilevato un backend bitmap. Si puo' forzare con `--graphics kitty` o `--graphics sixel`.

**Board fallback behavior**: la scacchiera Unicode viene sempre renderizzata sotto al PNG. Se il terminale ignora Kitty/SIXEL o il bitmap fallisce, il pannello non rimane vuoto.

**Graphics terminal handoff**: in `--graphics auto`, se il terminale corrente non supporta bitmap e l'app trova `kitty` o `wezterm` nel `PATH`, apre una nuova finestra separata e rilancia qchess con `--graphics kitty`. La finestra lanciata imposta un font size esplicito `14.0`. `QCHESS_GRAPHICS_CHILD=1` evita rilanci ricorsivi. Se non trova un terminale grafico, resta nel terminale corrente con fallback Unicode.

**Fritz assets**: tiles estratti da `fritz_3.png` con origine interna corretta `board_x=16`, `board_y=40`, `sq=40px`. Le coordinate `14,38` includono 2px di bordo e tagliano/spostano le figurine. Gli sprite non devono contenere pixel della casa: l'alpha va ricostruita usando sorgenti su casa chiara quando possibile, filtrando il tratteggio delle case scure e includendo solo le aree bianche chiuse dal contorno nero. `BoardStyle` + `PieceSet` traits per temi personalizzabili. Fritz tile inclusi con `include_bytes!` in `assets/fritz/`.

**Theme**: QBasic-inspired — Color::Blue background, black menu bar with yellow mnemonic labels, cyan borders, cyan selection highlight, dark grey status bar.

**State**: `AppScreen::Main` (normal) | `AppScreen::GamePicker` (floating overlay). Board always visible showing starting position when no game loaded.

**Why:** User wants retro feel inspired by Fritz/KnightStalker DOS chess software and QBasic IDE aesthetics.
**How to apply:** Keep these visual choices when adding new UI elements. New panels follow the same qblock() / Q_* color palette.
