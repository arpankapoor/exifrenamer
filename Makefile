PREFIX ?= ~/.local
BIN_PATH = $(PREFIX)/bin

build:
	cargo build --release

install: build
	install -d -m 755 $(BIN_PATH)
	install -m 755 ./target/release/exifrenamer $(BIN_PATH)

clean:
	cargo clean
