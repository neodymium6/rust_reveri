.PHONY: build test help dev run

# Python interpreter
PYTHON := python3
PYTEST := pytest
MATURIN := maturin

help:
	@echo "Available commands:"
	@echo "  make build    - Build the project with maturin (release mode)"
	@echo "  make dev      - Build and install in development mode"
	@echo "  make test     - Run tests"
	@echo "  make run      - Run the main.py script"

build:
	$(MATURIN) build -i $(PYTHON) --release

dev:
	$(MATURIN) develop

test:
	$(PYTEST) -v

run:
	$(PYTHON) main.py
