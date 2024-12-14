from rust_reversi import Arena  # type: ignore
import sys
import os

N_GAMES = 1000


def test_arena():
    python = sys.executable

    random_player = os.path.join(os.path.dirname(__file__), "random_player.py")

    arena = Arena([python, random_player], [python, random_player])

    arena.play_n(N_GAMES)

    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins1 + wins2 + draws == N_GAMES
    assert pieces1 + pieces2 > 0
