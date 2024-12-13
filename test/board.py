from rust_reversi import Board, Turn  # type: ignore


def test_init():
    board = Board()
    player_board, opponent_board, turn = board.get_board()
    assert player_board == 0x0000000810000000
    assert opponent_board == 0x0000001008000000
    assert turn == Turn.Black


def test_set_board():
    board = Board()
    player_board_t = 0x0000000000000000
    opponent_board_t = 0x0000000000000000
    turn_t = Turn.White
    board.set_board(player_board_t, opponent_board_t, turn_t)
    player_board, opponent_board, turn = board.get_board()
    assert player_board == player_board_t
    assert opponent_board == opponent_board_t
    assert turn == turn_t


def test_set_board_str():
    board = Board()
    board_str_t = "{}{}{}{}{}{}{}{}".format(
        "-" * 8,
        "-" * 8,
        "-" * 8,
        "-" * 8,
        "-" * 8,
        "-" * 8,
        "-" * 8,
        "-" * 8,
    )
    turn_t = Turn.White
    board.set_board_str(board_str_t, turn_t)

    player_board, opponent_board, turn = board.get_board()
    assert player_board == 0x0000000000000000
    assert opponent_board == 0x0000000000000000
    assert turn == turn_t
