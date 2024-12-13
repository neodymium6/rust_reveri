from rust_reversi import sum_as_string  # type: ignore


def test_add():
    a = 1
    b = 2
    assert sum_as_string(a, b) == "3"
