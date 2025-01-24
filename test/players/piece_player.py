import sys
from rust_reversi import Board, Turn, PieceEvaluator, AlphaBetaSearch
import random

DEPTH = int(sys.argv[1])
EPSILON = 1e-2


def main():
    turn = Turn.BLACK if sys.argv[2] == "BLACK" else Turn.WHITE
    board = Board()
    evaluator = PieceEvaluator()
    search = AlphaBetaSearch(evaluator, DEPTH, 1 << 10)

    while True:
        try:
            board_str = input().strip()

            if board_str == "ping":
                print("pong", flush=True)
                continue

            board.set_board_str(board_str, turn)
            if random.random() < EPSILON:
                move = board.get_random_move()
            else:
                move = search.get_move(board)

            print(move, flush=True)

        except Exception as e:
            print(e, file=sys.stderr)
            sys.exit(1)


if __name__ == "__main__":
    main()
