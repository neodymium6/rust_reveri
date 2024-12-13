from rust_reversi import Board, Turn, Color  # type: ignore


def main():
    board = Board()
    print(board)
    board.do_move(board.get_random_move())
    print(board)


if __name__ == "__main__":
    main()
