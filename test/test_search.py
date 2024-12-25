from rust_reversi import Arena
import sys
import os
import pytest

N_GAMES = 100

RANDOM_PLAYER = "players/random_player.py"
PIECE_PLAYER = "players/piece_player.py"
LEGAL_NUM_PLAYER = "players/legal_num_player.py"
CUSTOM_PLAYER = "players/custom_eval_player.py"
MATRIX_PLAYER = "players/matrix_player.py"


def get_player_path(filename: str) -> str:
    return os.path.join(os.path.dirname(__file__), filename)


def test_random_vs_piece():
    python = sys.executable
    random_player = get_player_path(RANDOM_PLAYER)
    piece_player = get_player_path(PIECE_PLAYER)
    depth = 2
    arena = Arena([python, random_player], [python, piece_player, str(depth)])
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins2 > wins1
    assert pieces2 > pieces1


def test_piece_depth_comparison():
    python = sys.executable
    piece_player = get_player_path(PIECE_PLAYER)
    depth1 = 2
    depth2 = 3
    arena = Arena(
        [python, piece_player, str(depth1)], [python, piece_player, str(depth2)]
    )
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins2 > wins1
    assert pieces2 > pieces1


def test_legal_num_vs_random():
    python = sys.executable
    random_player = get_player_path(RANDOM_PLAYER)
    legal_num_player = get_player_path(LEGAL_NUM_PLAYER)
    depth = 2
    arena = Arena([python, random_player], [python, legal_num_player, str(depth)])
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins2 > wins1
    assert pieces2 > pieces1


def test_invalid_depth():
    python = sys.executable
    legal_num_player = get_player_path(LEGAL_NUM_PLAYER)
    depth = -1
    arena = Arena(
        [python, legal_num_player, str(depth)],
        [python, legal_num_player, str(depth)],
    )
    with pytest.raises(Exception):
        arena.play_n(N_GAMES)


def test_custom_vs_piece():
    python = sys.executable
    custom_player = get_player_path(CUSTOM_PLAYER)
    piece_player = get_player_path(PIECE_PLAYER)
    depth = 2
    arena = Arena(
        [python, custom_player, str(depth)], [python, piece_player, str(depth)]
    )
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins1 > wins2
    assert pieces1 > pieces2


def test_matrix_vs_piece():
    python = sys.executable
    matrix_player = get_player_path(MATRIX_PLAYER)
    piece_player = get_player_path(PIECE_PLAYER)
    depth = 2
    arena = Arena(
        [python, matrix_player, str(depth)], [python, piece_player, str(depth)]
    )
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    pieces1, pieces2 = arena.get_pieces()

    assert wins1 > wins2
    assert pieces1 > pieces2


def test_custom_vs_matrix():
    python = sys.executable
    custom_player = get_player_path(CUSTOM_PLAYER)
    matrix_player = get_player_path(MATRIX_PLAYER)
    depth = 2
    arena = Arena(
        [python, custom_player, str(depth)], [python, matrix_player, str(depth)]
    )
    arena.play_n(N_GAMES)
    wins1, wins2, draws = arena.get_stats()
    win_ratio = abs((wins1 - wins2) / N_GAMES)

    assert win_ratio < 0.1  # sometimes it fails
