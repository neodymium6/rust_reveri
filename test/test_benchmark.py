import pytest
from rust_reversi import Board, Arena
import os
import sys

RANDOM_PLAYER = "players/random_player.py"


def get_player_path(filename: str) -> str:
    return os.path.join(os.path.dirname(__file__), filename)


def test_random_1000games(benchmark):
    def random_1000games():
        for _ in range(1000):
            board = Board()
            while not board.is_game_over():
                if board.is_pass():
                    board.do_pass()
                    continue
                board.do_move(board.get_random_move())

    benchmark(random_1000games)


def test_perft_8(benchmark):
    def perft_8():
        def perft(board: Board, depth: int) -> int:
            if depth == 0:
                return 1
            if board.is_game_over():
                return 1
            if board.is_pass():
                new_board = board.clone()
                new_board.do_pass()
                return perft(new_board, depth - 1)
            count = 0
            for new_board in board.get_child_boards():
                count += perft(new_board, depth - 1)
            return count

        board = Board()
        assert perft(board, 8) == 390216

    benchmark(perft_8)


def test_arena_1000games(benchmark):
    def arena_1000games():
        python = sys.executable
        random_player = get_player_path(RANDOM_PLAYER)
        arena = Arena([python, random_player], [python, random_player])
        arena.play_n(1000)

    benchmark(arena_1000games)
