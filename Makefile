

build:
	cargo wasm

test:
	cargo unit-test

artifacts: build 
	docker run --rm -v "$(shell pwd)":/code \
	--mount type=volume,source="$(shell basename "$(shell pwd)")_cache",target=/code/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/rust-optimizer:0.17.0

fmt:
	cargo fmt --all

clean:
	cargo clean

schema:
	cargo schema

.PHONY: build test artifacts fmt clean schema