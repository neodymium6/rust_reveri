from rust_reversi import Board

MAX_DEPTH = 8
ANS_MODE1 = [4, 12, 56, 244, 1396, 8200, 55092, 390216, 3005288, 24571284]
ANS_MODE2 = [4, 12, 56, 244, 1396, 8200, 55092, 390216, 3005320, 24571420]


def test_mode1():
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

    for i in range(1, MAX_DEPTH + 1):
        board = Board()
        assert perft(board, i) == ANS_MODE1[i - 1]


def test_mode2():
    def perft(board: Board, depth: int) -> int:
        if depth == 0:
            return 1
        if board.is_game_over():
            return 1
        if board.is_pass():
            new_board = board.clone()
            new_board.do_pass()
            return perft(new_board, depth)
        count = 0
        for new_board in board.get_child_boards():
            count += perft(new_board, depth - 1)
        return count

    for i in range(1, MAX_DEPTH + 1):
        board = Board()
        assert perft(board, i) == ANS_MODE2[i - 1]
