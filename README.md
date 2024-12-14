# Rust Reversi

A high-performance Reversi (Othello) game engine implemented in Rust with Python bindings. This library provides a fast and efficient Reversi implementation by leveraging Rust's performance while maintaining a friendly Python interface.

## Features

- High-performance implementation in Rust
- Efficient board representation using bitboards
- Easy-to-use Python interface
- Comprehensive game state manipulation methods
- Move generation and validation
- Random move sampling for testing
- Verified move generation through Perft testing

## Installation

```bash
pip install rust-reversi
```

## Basic Usage

```python
from rust_reversi import Board, Turn, Color

# Start a new game
board = Board()

# Display the current board state
print(board)

while not board.is_game_over():
    if board.is_pass():
        print("No legal moves available. Passing turn.")
        board.do_pass()
        continue

    # Get legal moves
    legal_moves = board.get_legal_moves_vec()
    print(f"Legal moves: {legal_moves}")

    # Get random move
    move = board.get_random_move()
    print(f"Random move: {move}")

    # Execute move
    board.do_move(move)
    print(board)

# Game over
winner = board.get_winner()
if winner is None:
    print("Game drawn.")
elif winner == Turn.BLACK:
    print("Black wins!")
else:
    print("White wins!")
```

## API Reference

### Classes

#### Turn

Represents a player's turn in the game.

- `Turn.BLACK`: Black player
- `Turn.WHITE`: White player

#### Color

Represents the state of a cell on the board.

- `Color.EMPTY`: Empty cell
- `Color.BLACK`: Black piece
- `Color.WHITE`: White piece

#### Board

The main game board class with all game logic.

##### Constructor

- `Board()`: Creates a new board with standard starting position

##### Board State Methods

- `get_board() -> tuple[int, int, Turn]`: Returns current board state (player bitboard, opponent bitboard, turn)
- `set_board(player_board: int, opponent_board: int, turn: Turn) -> None`: Sets board state directly
- `set_board_str(board_str: str, turn: Turn) -> None`: Sets board state from string representation
- `get_board_vec_black() -> list[Color]`: Returns flattened board state as if current player using black pieces
- `get_board_vec_turn() -> list[Color]`: Returns flattened board state with current player's pieces
- `get_board_matrix() -> list[list[list[int]]]`: Returns 3D matrix representation of board state
- `clone() -> Board`: Creates a deep copy of the board

##### Piece Count Methods

- `player_piece_num() -> int`: Returns number of current player's pieces
- `opponent_piece_num() -> int`: Returns number of opponent's pieces
- `black_piece_num() -> int`: Returns number of black pieces
- `white_piece_num() -> int`: Returns number of white pieces
- `piece_sum() -> int`: Returns total number of pieces on board
- `diff_piece_num() -> int`: Returns absolute difference in piece count

##### Move Generation and Validation

- `get_legal_moves() -> int`: Returns bitboard of legal moves
- `get_legal_moves_vec() -> list[int]`: Returns list of legal move positions
- `is_legal_move(pos: int) -> bool`: Checks if move at position is legal
- `get_random_move() -> int`: Returns random legal move position

##### Game State Methods

- `is_pass() -> bool`: Checks if current player must pass
- `is_game_over() -> bool`: Checks if game is finished
- `is_win() -> bool`: Checks if current player has won
- `is_lose() -> bool`: Checks if current player has lost
- `is_draw() -> bool`: Checks if game is drawn
- `is_black_win() -> bool`: Checks if black has won
- `is_white_win() -> bool`: Checks if white has won
- `get_winner() -> Optional[Turn]`: Returns winner of game (None if draw)

##### Move Execution

- `do_move(pos: int) -> None`: Executes move at specified position
- `do_pass() -> None`: Executes pass move when no legal moves available

##### Board Representation

- `__str__() -> str`: Returns string representation of board

Board is displayed as:

```text
 |abcdefgh
-+--------
1|XXXXXXXX
2|OOOOOOOO
3|--------
...
```

Where:

- `X`: Black pieces
- `O`: White pieces
- `-`: Empty cells

## Development

### Requirements

- Python >=3.8
- Rust toolchain

### Building from Source

```bash
git clone https://github.com/yourusername/rust-reversi.git
cd rust-reversi
make dev    # Development build
```

### Running Tests

```bash
make test
```

### Available Commands

- `make build`: Build the project in release mode
- `make dev`: Build and install in development mode
- `make test`: Run test suite
- `make run`: Run example script

## Testing

The project includes comprehensive test coverage including Perft (Performance Test) for verifying game tree correctness:

### Perft Testing

The Perft (performance test) suite verifies the correctness of the move generator by counting all possible game positions at different depths. This ensures:

- Legal move generation is working correctly
- Game state transitions are handled properly
- All game tree paths are being correctly explored

Two testing modes are implemented:

1. Standard mode: Counts leaf nodes at each depth
1. Pass-exclusive mode: Counts leaf nodes. Depth does not decriment by passing turn

These tests compare against known correct node counts for the Reversi game tree, providing confidence in the game engine's core functionality.

## Performance

The library uses bitboard representation and efficient algorithms for:

- Legal move generation
- Board state updates
- Game state evaluation
