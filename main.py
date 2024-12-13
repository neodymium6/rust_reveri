from rust_reversi import Board, Turn, Color  # type: ignore


def main():
    board = Board()
    print(board.get_board())
    print(board.get_board_vec())
    print(Color.Black)


if __name__ == "__main__":
    main()
