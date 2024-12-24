from rust_reversi import Arena
import sys
import os

N_GAMES = 1000
TEST_PORT = 12345

RANDOM_PLAYER = "players/random_player.py"
PIECE_PLAYER = "players/piece_player.py"


def get_player_path(filename: str) -> str:
    return os.path.join(os.path.dirname(__file__), filename)


def test_random_vs_piece():
    python = sys.executable
    random_player = get_player_path(RANDOM_PLAYER)
    piece_player = get_player_path(PIECE_PLAYER)
    arena = Arena([python, random_player], [python, piece_player])
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins2 > wins1
    assert pieces2 > pieces1
