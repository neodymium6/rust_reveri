from rust_reversi import Counter  # type: ignore


def test_increment():
    counter = Counter()
    counter.increment()
    assert counter.get_count() == 1


def test_str():
    counter = Counter()
    assert str(counter) == "Counter(0)"
    counter.increment()
    assert str(counter) == "Counter(1)"
