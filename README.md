# DEP

Rust implementation for the DEP contract calls.

## Install

Clone the repo.

``` shell
git clone https://github.com/deeper-chain/web3d.git
```

## Run tests

``` shell
cd web3d
cargo test
```

## Basic usage

```rust
// Your local eth wallet, for paying gas fees
let wallet = LocalWallet::decrypt_keystore("./eth.keystore", "VGPUmPKNtBzDvCJK").unwrap();
let client = Client::new(
    // The chain rpc endpoint
    "https://mainnet-dev.deeper.network/rpc",
    // DEP contract address, remove the prefix 0x
    "9397AA12576cEc2A37C60f76d2FB31b31b5E5c7F",
    // DEP contract abi.json, can be found in the code tab on the blockscout page
    "./testnet.json",
    wallet,
)
.unwrap();
println!("{:?}", client.task_info(1).await.unwrap());
```