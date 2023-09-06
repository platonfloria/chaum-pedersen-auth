-include .env
export $(shell sed 's/=.*//' .env)

build:
	mkdir -p ./generated
	cargo build --release --target wasm32-unknown-unknown
	trunk build --release

run:
	mkdir -p ./generated
	trunk serve

test:
	cargo test
