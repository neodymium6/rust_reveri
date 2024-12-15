.PHONY: build test help dev run bench bench-save bench-comp bench-repo requirements install

# Python interpreter
PYTHON := python3
PYTEST := pytest
MATURIN := maturin
PIP := pip

help:
	@echo "Available commands:"
	@echo "  make help         - Show this message"
	@echo "  make requirements - Save the current environment to requirements.txt"
	@echo "  make install      - Install the project dependencies"
	@echo "  make build        - Build the project with maturin (release mode)"
	@echo "  make dev          - Build and install in development mode"
	@echo "  make test         - Run tests"
	@echo "  make run          - Run the main.py script"
	@echo "  make bench        - Run benchmarks"
	@echo "  make bench-save   - Run benchmarks and save results"
	@echo "  make bench-comp   - Run benchmarks and compare with previous results"
	@echo "  make bench-repo   - Generate a report with the benchmark results, and update the README.md file"

build:
	$(MATURIN) build -i $(PYTHON) --release

dev:
	$(MATURIN) develop

test:
	$(PYTEST) -v --benchmark-skip

run:
	$(PYTHON) main.py

bench:
	$(MATURIN) develop --release
	$(PYTEST) -v --benchmark-only

bench-save:
	$(MATURIN) develop --release
	$(PYTEST) -v --benchmark-autosave --benchmark-only

bench-comp:
	$(MATURIN) develop --release
	$(PYTEST) -v --benchmark-compare --benchmark-only

bench-repo:
	$(PYTHON) bench_report.py

requirements:
	$(PIP) freeze | grep -v rust_reversi > requirements.txt

install:
	$(PIP) install -r requirements.txt
	$(MATURIN) develop
