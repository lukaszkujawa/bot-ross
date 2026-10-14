.PHONY: build run test bench demo sample sketch

REFERENCE ?= examples/reference.png
CONFIG ?= examples/quick.toml
SIZE ?= 512
OUT ?= out.png
BRUSH ?= brushes/watercolour.toml

build:
	cargo build --release

run:
	cargo run --release -- $(REFERENCE) --size $(SIZE) --config $(CONFIG) --out $(OUT)

test:
	cargo test

bench:
	cargo run --release --example bench

demo:
	cargo run --release --example demo

sample:
	cargo run --release --example sample

sketch:
	cargo run --release --bin sketch -- --brush $(BRUSH)
