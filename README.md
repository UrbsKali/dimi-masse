# STM32H723ZGT Template
This template can be used to quickstart your embed Rust projet on this board

# Setup
(Install Rust before)
```sh
$ rustup update
$ rustup component add llvm-tools
$ cargo install cargo-binutils
$ cargo install cargo-embed (--force if needed)
$ cargo install cargo-generate
$ rustup target add thumbv7em-none-eabih
```

# Usage
```sh
$ cargo generate --git https://github.com/UrbsKali/rust-stm32h723zgt-template
$ cargo embed
```