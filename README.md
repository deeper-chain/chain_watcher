# chain_watcher

The deeper chain watcher application. Manages the tasks of deeper chain.

## Install

Clone the repo and make a release of the package:

``` shell
git https://github.com/deeper-chain/chain_watcher.git
git checkout -b gc origin/gc
cd chain_watcher
```

To build this project, make sure you have cross compiler tools installed on your development machine.

### MacOS

macOS cross compiler toolchains, supports both Apple Silicon & Intel Macs.

install using Homebrew:

```bash
brew tap messense/macos-cross-toolchains
# install x86_64-unknown-linux-gnu toolchain
brew install x86_64-unknown-linux-gnu
# install aarch64-unknown-linux-gnu toolchain
brew install aarch64-unknown-linux-gnu
```

Suppose you have installed `x86_64-unknown-linux-gnu` toolchain and have it on `PATH`,
setup the environment variables as below to use it with Cargo.

```bash
export CC_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-gcc
export CXX_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-g++
export AR_x86_64_unknown_linux_gnu=x86_64-unknown-linux-gnu-ar
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc
rustup target add x86_64-unknown-linux-gnu
cd chain_watcher
cargo build --target=x86_64-unknown-linux-gnu --release
```

for aarch64

```bash
export CC_aarch64_unknown_linux_gnu=aarch64-unknown-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-unknown-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-unknown-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-unknown-linux-gnu-gcc
rustup target add aarch64-unknown-linux-gnu
cd chain_watcher
cargo build --target=aarch64-unknown-linux-gnu --release
```