from rust_reversi import Board, Turn  # type: ignore


def main():
    # Start a new game
    board = Board()

    # Display the current board state
    print(board)

    while not board.is_game_over():
        if board.is_pass():
            print("No legal moves available. Passing turn.")
            board.do_pass()
            continue

        # Get legal moves
        legal_moves = board.get_legal_moves_vec()
        print(f"Legal moves: {legal_moves}")

        # Get random move
        move = board.get_random_move()
        print(f"Random move: {move}")

        # Execute move
        board.do_move(move)
        print(board)

    # Game over
    winner = board.get_winner()
    if winner is None:
        print("Game drawn.")
    elif winner == Turn.BLACK:
        print("Black wins!")
    else:
        print("White wins!")


if __name__ == "__main__":
    main()
