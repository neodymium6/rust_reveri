import threading
import time
from rust_reversi import Arena, NetworkArenaServer, NetworkArenaClient
import sys
import os
import pytest

N_GAMES = 1000
TEST_PORT = 12345

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

    with pytest.raises(ValueError, match="Game error: (Black|White)Timeout"):
        arena.play_n(2)


def test_network_arena_basic():
    """Test basic functionality of network arena with random players"""
    python = sys.executable
    random_player = get_player_path(RANDOM_PLAYER)

    # Start server
    server = NetworkArenaServer(N_GAMES)
    server_thread = threading.Thread(
        target=lambda: server.start("localhost", TEST_PORT)
    )
    server_thread.daemon = True
    server_thread.start()
    time.sleep(0.1)  # Wait for server to start

    # Create and connect two clients
    client1 = NetworkArenaClient([python, random_player])
    client2 = NetworkArenaClient([python, random_player])

    # Start clients in separate threads
    client1_thread = threading.Thread(
        target=lambda: client1.connect("localhost", TEST_PORT)
    )
    client2_thread = threading.Thread(
        target=lambda: client2.connect("localhost", TEST_PORT)
    )

    client1_thread.start()
    client2_thread.start()

    # Wait for clients to finish
    client1_thread.join()
    client2_thread.join()

    # Check results
    wins1, losses1, draws1 = client1.get_stats()
    pieces1, opponent_pieces1 = client1.get_pieces()

    assert wins1 + losses1 + draws1 == N_GAMES
    assert pieces1 + opponent_pieces1 > 0
    win_ratio = abs((wins1 - losses1) / N_GAMES)
    assert win_ratio < 0.1  # Allow for some randomness


def test_network_arena_invalid_game_count():
    """Test that server creation fails with odd game count"""
    with pytest.raises(ValueError, match="Game count must be even"):
        NetworkArenaServer(99)


def test_network_arena_invalid_player():
    """Test behavior with invalid player executable"""
    python = sys.executable
    invalid_player = get_player_path(NONEXISTENT_PLAYER)

    server = NetworkArenaServer(N_GAMES)
    server_thread = threading.Thread(
        target=lambda: server.start("localhost", TEST_PORT)
    )
    server_thread.daemon = True
    server_thread.start()
    time.sleep(0.1)  # Wait for server to start

    client = NetworkArenaClient([python, invalid_player])

    with pytest.raises(ValueError, match="ping-pong test failed"):
        client.connect("localhost", TEST_PORT)
