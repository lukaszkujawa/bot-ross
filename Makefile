.PHONY: build run test bench demo strokes sample sketch mcp

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
	cargo test --workspace

bench:
	cargo run --release --example bench

DEMOS := $(patsubst %.toml,%.jpg,$(wildcard demo/demo*.toml))

demo: $(DEMOS)

demo/demo%.jpg: demo/demo%.toml demo/ref%.jpg
	cargo run --release -- demo/ref$*.jpg --size 1024 --config $< --out $@ --headless

strokes:
	cargo run --release --example demo

sample:
	cargo run --release --example sample

sketch:
	cargo run --release --bin sketch -- --brush $(BRUSH)

mcp:
	@cargo run --release -q -p bot-ross-mcp -- --brushes brushes
