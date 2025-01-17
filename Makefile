VERSION = $(patsubst "%",%, $(word 3, $(shell grep version Cargo.toml)))
BUILD_TIME = $(shell date +"%Y/%m/%d %H:%M:%S")
GIT_REVISION = $(shell git log -1 --format="%h")
RUST_VERSION = $(word 2, $(shell rustc -V))
LONG_VERSION = "$(VERSION) ( rev: $(GIT_REVISION), rustc: $(RUST_VERSION), build at: $(BUILD_TIME) )"
BIN_NAME = procs

export LONG_VERSION

.PHONY: all test clean release_lnx release_win release_mac

all: test

test:
	cargo test --locked

watch:
	cargo watch test --locked

clean:
	cargo clean

release_lnx:
	cargo build --locked --release --target=x86_64-unknown-linux-musl
	zip -j ${BIN_NAME}-v${VERSION}-x86_64-linux.zip target/x86_64-unknown-linux-musl/release/${BIN_NAME}

release_lnx_aarch64:
	cargo build --locked --release --target=aarch64-unknown-linux-musl
	zip -j ${BIN_NAME}-v${VERSION}-aarch64-linux.zip target/aarch64-unknown-linux-musl/release/${BIN_NAME}

release_win:
	cargo build --locked --release --target=x86_64-pc-windows-msvc
	mv -v target/x86_64-pc-windows-msvc/release/${BIN_NAME}.exe ./
	7z a ${BIN_NAME}-v${VERSION}-x86_64-windows.zip ${BIN_NAME}.exe

release_mac:
	cargo build --locked --release --target=x86_64-apple-darwin
	cargo build --locked --release --target=aarch64-apple-darwin
	zip -j ${BIN_NAME}-v${VERSION}-x86_64-mac.zip target/x86_64-apple-darwin/release/${BIN_NAME}
	zip -j ${BIN_NAME}-v${VERSION}-aarch64-mac.zip target/aarch64-apple-darwin/release/${BIN_NAME}

release_rpm:
	mkdir -p target
	cargo rpm build
	cp target/x86_64-unknown-linux-musl/release/rpmbuild/RPMS/x86_64/* ./
