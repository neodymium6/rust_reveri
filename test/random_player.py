import sys
from rust_reversi import Board, Turn  # type: ignore


def main():
    turn = Turn.BLACK if sys.argv[1] == "BLACK" else Turn.WHITE
    board = Board()

    while True:
        try:
            board_str = input().strip()
            board.set_board_str(board_str, turn)

            move = board.get_random_move()

            print(move, flush=True)

        except Exception as e:
            print(e, file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
