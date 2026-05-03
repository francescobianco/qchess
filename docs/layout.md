# Layout

## Main Screen

The base layout is:

1. Menu bar at the top.
2. Chessboard always visible on the left below the menu bar.
3. Move list to the right of the chessboard.
4. Engine analysis output below the chessboard and move list.
5. Status bar at the bottom.

The chessboard must remain visible whether or not a game or database is open.

## State Model

The application keeps two current objects:

- Current database.
- Current game.

Games are processed one at a time. Changing games happens through the Games menu and database open/search flows.
