# web3d

The deeper chain watcher application. Manages the tasks of deeper chain.

## Install

Clone the repo and make a release of the package:

``` shell
git clone https://github.com/deeper-chain/web3d.git
cd web3d
git checkout -b gc origin/gc
cargo build --release
```

To build this project for other platform, make sure you have cross compiler tools installed on your development machine.

### host MacOS, target for arm

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
cd web3d
cargo build --target=x86_64-unknown-linux-gnu --release
```

for aarch64

```bash
export CC_aarch64_unknown_linux_gnu=aarch64-unknown-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-unknown-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-unknown-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-unknown-linux-gnu-gcc
rustup target add aarch64-unknown-linux-gnu
cd web3d
cargo build --target=aarch64-unknown-linux-gnu --release
```

### host x86 Linux, target arm64

intall cross compiler toolchain and dependency library

```bash
sudo apt install -y gcc-aarch64-linux-gnu
sudo apt install -y pkg-config
sudo apt install -y libssl-dev
```


```bash
rustup target add aarch64-unknown-linux-gnu
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
cd web3d
cargo build --target=aarch64-unknown-linux-gnu --release
```

## Systemd configuration
```bash
[Unit]
Description=Chain watcher
[Service]
Type=simple
ExecStartPre=mkdir -p {{your log stash directory path}}
ExecStart={{built binary path}}
ExecStop=pkill watcher
Restart=always
RestartSec=5
StandardOutput=append:{{your std out log path}}
StandardError=append:{{your std err log path}}
[Install]
WantedBy=multi-user.target
```
