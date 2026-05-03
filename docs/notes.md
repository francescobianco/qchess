---
name: qchess project overview
description: TUI chess app in Rust — architecture, stack, design decisions
type: project
originSessionId: b53f3c15-ba19-48ea-8891-2c31ff392650
---
Rust TUI chess database browser. Any folder with .pgn files is a valid database.

**Stack**: ratatui 0.29, crossterm 0.28, shakmaty 0.28, pgn-reader 0.27, walkdir 2, image 0.24 (0.25 requires Rust 1.88; installed toolchain is 1.86), base64 0.22

**Board rendering**: PNG via automatic Kitty/SIXEL backend selection. Kitty is used on kitty/WezTerm; SIXEL is selected on VTE/GNOME Terminal when detectable, so the PNG board can work on Ubuntu GNOME. Unicode fallback is available with `--unicode`, `--graphics unicode`, or when no bitmap backend is detected. Backends can be forced with `--graphics kitty` or `--graphics sixel`.

**Board fallback behavior**: the Unicode board is always rendered below the PNG. If the terminal ignores Kitty/SIXEL or bitmap drawing fails, the board area is not blank.

**Board cursor**: arrow keys move `App::selected_square` on the chessboard. In the PNG renderer the current square has a solid 2 px blue border drawn above the square and piece; in the Unicode fallback the current square uses a blue background.

**Graphics terminal handoff**: in `--graphics auto`, if the current terminal does not support bitmap rendering and the app finds `kitty` or `wezterm` in `PATH`, it opens a separate window and relaunches qchess with `--graphics kitty`. The launched window uses explicit font size `14.0`. `QCHESS_GRAPHICS_CHILD=1` prevents recursive relaunches. If no graphics-capable terminal is found, qchess stays in the current terminal with Unicode fallback.

**Fritz assets**: tiles are extracted from `fritz_3.png` with corrected internal origin `board_x=16`, `board_y=40`, `sq=40px`. Coordinates `14,38` include 2 px of border and cut/shift the pieces. Piece sprites must not contain square pixels: alpha is reconstructed from light-square sources when possible, dark-square hatch is filtered, and only white areas enclosed by the black outline are included. `BoardStyle` + `PieceSet` traits support custom themes. Fritz tiles are embedded with `include_bytes!` from `assets/fritz/`.

**Theme**: current style from `docs/style.md` — general white background, black menu bar with white text, no bordered boxes around the board or move list, blue active square cursor, black status bar. The engine analysis area uses a horizontal ASCII separator line with title, not a bordered box.

**Fallback board geometry**: Unicode board uses a 1-column rank label strip directly adjacent to the board, 8 squares at 4x2 terminal cells, one bottom coordinate bar, and a 1-column black separator on the right before the move list. Total board widget size is 34x17. The top row is fixed at board height, so no empty rows appear between board coordinates and the engine analysis separator.

**State**: `AppScreen::Main` (normal) | `AppScreen::GamePicker` (floating overlay) | `AppScreen::EngineMenu` (menu-bar dropdown) | `AppScreen::EngineEditor` (engine form overlay). Board always visible showing starting position when no game is loaded.

**Engine menu**: `E` opens the Engine dropdown below the `Engine` menu-bar entry. The dropdown shows `Add New Engine`, a separator, and registered engines with an active flag. Selecting an engine opens the editor. Engine config persists to `$HOME/.qchess.toml`.

**Why:** User wants retro chess visuals inspired by Fritz/KnightStalker assets, but the application chrome is no longer QBasic blue.
**How to apply:** Keep new UI elements on white background with black structural chrome. The menu stays black; engine analysis uses a title separator bar.
