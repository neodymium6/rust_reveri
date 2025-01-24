# Rust Reversi

A high-performance Reversi (Othello) game engine implemented in Rust with Python bindings. This library provides a fast and efficient Reversi implementation by leveraging Rust's performance while maintaining a friendly Python interface.

Core implementation is based on [rust_reversi_core](https://github.com/neodymium6/rust_reversi_core).

## Features

- High-performance implementation in Rust
- Efficient board representation using bitboards
- Easy-to-use Python interface
- Comprehensive game state manipulation methods
- Move generation and validation
- Random move sampling for testing
- Verified move generation through Perft testing
- Alpha-beta pruning based search with customizable evaluation functions
- Arena system for AI player evaluation
  - Local arena for direct player evaluation
  - Network arena for distributed evaluation
- Process-based player execution with timeout management
- Fair player evaluation with color alternation

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

### Using the Local Arena

The Local Arena allows you to evaluate your own AI players by running them locally on the same machine. This is useful for testing and comparing different versions or strategies of your AI implementations:

```python
from rust_reversi import Arena
import sys

# Create an arena with two AI players
python = sys.executable
player1 = ["python", "player1.py"]  # Command to run first player
player2 = ["./player2"]             # Command to run second player

# Initialize the arena
arena = Arena(player1, player2)

# Play 100 games (must be an even number for fair color distribution)
arena.play_n(100)

# Get statistics
wins1, wins2, draws = arena.get_stats()
print(f"Player 1 wins: {wins1}")
print(f"Player 2 wins: {wins2}")
print(f"Draws: {draws}")

# Get total pieces captured
pieces1, pieces2 = arena.get_pieces()
print(f"Player 1 total pieces: {pieces1}")
print(f"Player 2 total pieces: {pieces2}")
```

### Using the Network Arena

The Network Arena provides a system for playing against other people's AIs over a network. This allows for competitive matches and tournaments between different developers' AIs:

```python
# Server side
from rust_reversi import NetworkArenaServer

# Create a server that will run 1000 games per session
server = NetworkArenaServer(1000)

# Start the server
server.start("localhost", 12345)
```

```python
# Client side
from rust_reversi import NetworkArenaClient
import sys

# Create a client with an AI player
client = NetworkArenaClient(["python", "player.py"])

# Connect to the server
client.connect("localhost", 12345)

# Get results after games
wins, losses, draws = client.get_stats()
pieces_captured, opponent_pieces = client.get_pieces()
print(f"Wins: {wins}, Losses: {losses}, Draws: {draws}")
print(f"Pieces Captured: {pieces_captured}, Opponent Pieces: {opponent_pieces}")
```

### Creating AI Players

AI players should be implemented as scripts that:

1. Accept a command line argument specifying their color ("BLACK" or "WHITE")
2. Read board states from stdin
3. Write moves to stdout
4. Handle the "ping"/"pong" protocol for connection verification

Example player implementation with alpha-beta search:

```python
import sys
from rust_reversi import Board, Turn, PieceEvaluator, AlphaBetaSearch

# Maximum search depth
DEPTH = 3

def main():
    # Get color from command line argument
    turn = Turn.BLACK if sys.argv[1] == "BLACK" else Turn.WHITE
    board = Board()
    
    # Initialize evaluator and search
    evaluator = PieceEvaluator()
    search = AlphaBetaSearch(evaluator, DEPTH, win_score=1 << 10)

    while True:
        try:
            board_str = input().strip()

            # Handle ping/pong protocol
            if board_str == "ping":
                print("pong", flush=True)
                continue

            # Update board state
            board.set_board_str(board_str, turn)
            
            # Get and send move using alpha-beta search
            move = search.get_move(board)
            print(move, flush=True)

        except Exception as e:
            print(e, file=sys.stderr)
            sys.exit(1)

if __name__ == "__main__":
    main()
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

##### Board Constructor

- `Board()`: Creates a new board with standard starting position

##### Board State Methods

- `get_board() -> tuple[int, int, Turn]`: Returns current board state (player bitboard, opponent bitboard, turn)
- `set_board(player_board: int, opponent_board: int, turn: Turn) -> None`: Sets board state directly
- `set_board_str(board_str: str, turn: Turn) -> None`: Sets board state from string representation
- `get_board_src() -> str`: Returns string representation of board state
- `get_board_vec_black() -> list[Color]`: Returns flattened board state as if current player using black pieces
- `get_board_vec_turn() -> list[Color]`: Returns flattened board state with current player's pieces
- `get_board_matrix() -> list[list[list[int]]]`: Returns 3D matrix representation of board state
- `get_child_boards() -> list[Board]`: Returns list of child boards for all legal moves
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
- `get_legal_moves_tf() -> list[bool]`: Returns list of legal move positions as boolean mask
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

#### Search and Evaluation Classes

##### Evaluator (Base Class)

Base class for board evaluation functions.
Extend this class to implement custom evaluation functions.
set_py_evaluator() method can be used to set a Python evaluator class for evaluation.
See test/players/custom_eval_player.py for an example.

###### Evaluator Constructor

- `Evaluator()`: Creates a new evaluator

###### Evaluator Methods

- `evaluate(board: Board) -> int`: Evaluates the given board state.
override this method in subclasses to implement custom evaluation functions.

- `set_py_evaluator(Evaluator) -> None`: Sets a Python evaluator class for evaluation

##### PieceEvaluator (extends Evaluator)

Simple evaluator that uses piece difference for evaluation.

###### PieceEvaluator Constructor

- `PieceEvaluator()`: Creates a new piece-counting evaluator

##### LegalNumEvaluator (extends Evaluator)

Evaluator that uses number of legal moves for evaluation.

###### LegalNumEvaluator Constructor

- `LegalNumEvaluator()`: Creates a new legal-moves-counting evaluator

##### MatrixEvaluator (extends Evaluator)

Evaluator that uses a matrix of weights for evaluation.
Use 8x8 matrix for evaluation.
Scores are calculated as product of matrix weights and board state. (player equals 1, opponent equals -1)

###### MatrixEvaluator Constructor

- `MatrixEvaluator(matrix: List[List[int])`: Creates a new matrix-based evaluator with given weights

###### WinrateEvaluator Constructor

- `WinrateEvaluator()`: Creates a new evaluator that predicts winrate.

###### WinrateEvaluator Methods

- `evaluate(board: Board) -> int`: Evaluates the given board state.
override this method in subclasses to implement custom evaluation functions.

- `set_py_evaluator(WinrateEvaluator) -> None`: Sets a Python evaluator class for evaluation

##### AlphaBetaSearch

Alpha-beta pruning based search for finding best moves.

###### AlphaBetaSearch Constructor

- `AlphaBetaSearch(evaluator: Evaluator, depth: int)`: Creates a new search instance with given evaluator and search depth

###### AlphaBetaSearch Methods

- `get_move(board: Board) -> int`: Returns best move found within specified depth
- `get_move_with_timeout(board: Board, timeout_ms: int) -> int`: Returns best move found with iterative deepening up to timeout in milliseconds
- `get_search_score(board: Board) -> int`: Returns search score for current board state

##### ThunderSearch

###### ThunderSearch Constructor

- `ThunderSearch(evaluator: WinrateEvaluator, n_playouts: int, epsilon: float)`: Creates a new search instance with given evaluator, number of playouts, and epsilon value

###### ThunderSearch Methods

- `get_move(board: Board) -> int`: Returns best move found within specified playouts
- `get_move_with_timeout(board: Board, timeout_ms: int) -> int`: Returns best move found up to timeout in milliseconds
- `get_search_score(board: Board) -> int`: Returns search score for current board state

#### Arena Classes

##### Local Arena

The Arena class manages local matches between two AI players.

###### Arena Constructor

- `Arena(command1: List[str], command2: List[str])`: Creates a new arena with commands to run two players

###### Arena Methods

- `play_n(n: int) -> None`: Play n games between the players (n must be even)
- `get_stats() -> Tuple[int, int, int]`: Returns (player1_wins, player2_wins, draws)
- `get_pieces() -> Tuple[int, int]`: Returns total pieces captured by each player

##### Network Arena Server

The NetworkArenaServer class manages distributed matches between players connecting over network.

###### NetworkArenaServer Constructor

- `NetworkArenaServer(games_per_session: int)`: Creates a new server that runs specified number of games per session

###### NetworkArenaServer Methods

- `start(address: str, port: int) -> None`: Starts the server on specified address and port

##### Network Arena Client

The NetworkArenaClient class allows AI players to connect to a network arena server.

###### NetworkArenaClient Constructor

- `NetworkArenaClient(command: List[str])`: Creates a new client with command to run the player

###### NetworkArenaClient Methods

- `connect(address: str, port: int) -> None`: Connects to server at specified address and port
- `get_stats() -> Tuple[int, int, int]`: Returns (wins, losses, draws)
- `get_pieces() -> Tuple[int, int]`: Returns total pieces captured by player and opponent

## Development

### Requirements

- Python >=3.8
- Rust toolchain

### Building from Source

```bash
git clone https://github.com/neodymium6/rust_reversi.git
cd rust_reversi

# Create and activate virtual environment (recommended)
python -m venv .venv
source .venv/bin/activate

# Install dependencies
make install

# Or for development setup
pip install -r requirements.txt
make dev
```

### Available Commands

- `make help`: Show available commands
- `make requirements`: Save current dependencies to requirements.txt
- `make install`: Install the project dependencies
- `make build`: Build the project with maturin (release mode)
- `make dev`: Build and install the project in development mode
- `make test`: Run tests
- `make run`: Run the main.py script
- `make bench`: Run benchmarks
- `make bench-save`: Run benchmarks and save results
- `make bench-comp`: Run benchmarks and compare with previous saved results
- `make bench-repo`: Generate a report with benchmark results, and update the README

## Testing

The project includes comprehensive test coverage including:

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

## Benchmark Results

Benchmark history from 2024-12-15 to 2025-01-24

### Summary

| Test | Current | Min (Historical) | Max (Historical) | Trend |
|------|---------|-----------------|------------------|-------|
| Random 1000Games | 17.27ms | 17.27ms | 23.45ms | 📈 Improved |
| Perft 8 | 68.03ms | 68.03ms | 115.85ms | 📈 Improved |
| Arena 1000Games | 1.01s | 870.07ms | 1.63s | 📈 Improved |

### Latest System Information

- CPU: Apple M1
- Architecture: arm64
- Cores: 8
- Python: 3.9.21

### Performance History

![Performance History](./docs/images/benchmark_history.svg)

### Operations Per Second History

![Operations History](./docs/images/benchmark_ops_history.svg)

*Note: Performance may vary based on system specifications and load.*
