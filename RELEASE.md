# Release

NOTE: these steps were done on Ubuntu 20.04

### Source
```
git checkout main
git pull
git show
```

### Setup

Skip this section if you have already setup dependencies for deploying Dank.

If you are missing any packages:
```
apt-get update -y
apt-get install -y libssl-dev pkg-config
```

If you haven't installed the wasm32 target, install it with
```
rustup target add wasm32-unknown-unknown
```

If you haven't installed the IC CDK optimizer, install it with the following. Run this from a directory where there is no Cargo.toml:
```
cargo install ic-cdk-optimizer
```

### Prepare

If cargo.lock file has been changed, run
```
cargo update
```

Build and test
```
node build.js
cargo test
```

### Backup
```
cd ./test/smoke
yarn
yarn backup:live
```

### Deploy
```
dfx canister --network=ic --wallet $(dfx identity --network=ic get-wallet) call xtc halt
backup again
dfx canister --network=ic install --mode=upgrade xtc
```

### Smoke tets

Balance
```
dfx canister --network=ic call xtc  balance "(null)"
```

Transfer
```
dfx canister --network=ic --no-wallet call xtc transfer "(record { to= principal \"some-account's-principal-id\"; amount= (1000:nat64) })"
```

Stats
```
dfx canister --network=ic call xtc stats
```
