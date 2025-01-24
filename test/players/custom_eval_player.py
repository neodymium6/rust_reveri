import sys
from rust_reversi import Board, Turn, Evaluator, AlphaBetaSearch, Color
import random

DEPTH = int(sys.argv[1])
EPSILON = 1e-2

EVAL_MATRIX = [
    [50, -10, 11, 6, 6, 11, -10, 50],
    [-10, -15, 1, 2, 2, 1, -15, -10],
    [11, 1, 1, 1, 1, 1, 1, 11],
    [6, 2, 1, 3, 3, 1, 2, 6],
    [6, 2, 1, 3, 3, 1, 2, 6],
    [11, 1, 1, 1, 1, 1, 1, 11],
    [-10, -15, 1, 2, 2, 1, -15, -10],
    [50, -10, 11, 6, 6, 11, -10, 50],
]


class CustomEvaluator(Evaluator):
    def __init__(self):
        super().set_py_evaluator(self)

    def evaluate(self, board: Board) -> int:
        board_vec = board.get_board_vec_black()
        score = 0
        for i in range(8):
            for j in range(8):
                if board_vec[i * 8 + j] == Color.BLACK:
                    score += EVAL_MATRIX[i][j]
                elif board_vec[i * 8 + j] == Color.WHITE:
                    score -= EVAL_MATRIX[i][j]
        return score


def main():
    turn = Turn.BLACK if sys.argv[2] == "BLACK" else Turn.WHITE
    board = Board()
    evaluator = CustomEvaluator()
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
