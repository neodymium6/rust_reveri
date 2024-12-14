from rust_reversi import Arena  # type: ignore
import sys
import os
import pytest

N_GAMES = 1000

RANDOM_PLAYER = "players/random_player.py"
NONEXISTENT_PLAYER = "players/nonexistent_player.py"
SLOW_PLAYER = "players/slow_player.py"


def get_player_path(filename: str) -> str:
    return os.path.join(os.path.dirname(__file__), filename)


def test_random_vs_random():
    python = sys.executable
    random_player = get_player_path(RANDOM_PLAYER)
    arena = Arena([python, random_player], [python, random_player])
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins1 + wins2 + draws == N_GAMES
    assert pieces1 + pieces2 > 0
    win_ratio = abs((wins1 - wins2) / N_GAMES)
    assert win_ratio < 0.1  # sometimes it fails


def test_arena_odd_games():
    python = sys.executable
    random_player = get_player_path(RANDOM_PLAYER)
    arena = Arena([python, random_player], [python, random_player])

    with pytest.raises(ValueError, match="Game count must be even"):
        arena.play_n(999)


def test_arena_invalid_player():
    python = sys.executable
    invalid_player = get_player_path(NONEXISTENT_PLAYER)
    arena = Arena([python, invalid_player], [python, invalid_player])

    with pytest.raises(ValueError, match="Engine start error"):
        arena.play_n(2)


def test_arena_multiple_sessions():
    python = sys.executable
    random_player = get_player_path(RANDOM_PLAYER)
    arena = Arena([python, random_player], [python, random_player])

    arena.play_n(100)
    first_stats = arena.get_stats()

    arena.play_n(100)
    second_stats = arena.get_stats()

    assert sum(second_stats) == 200
    assert all(b >= a for a, b in zip(first_stats, second_stats))


def test_arena_timeout():
    python = sys.executable
    slow_player = get_player_path(SLOW_PLAYER)
    random_player = get_player_path(RANDOM_PLAYER)

    arena = Arena([python, slow_player], [python, random_player])

    with pytest.raises(ValueError, match="Game error: BlackTimeout"):
        arena.play_n(2)
