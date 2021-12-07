![Dank Logo](https://storageapi.fleek.co/fleek-team-bucket/Dank/Banner.png)

# Dank - The Internet Computer Decentralized Bank

Dank is a collection of Open Internet Services for users and developers on the Internet Computer. In this repository, you will find the codebase for Dank's projects, including the canisters that provide these services.

- [Official Website](https://dank.ooo/) 
- [Twitter Handle](https://twitter.com/dank_ois)
- [Security & Issue Reporting Policy](https://github.com/Psychedelic/dank/security/policy) 

## Main Products

### Cycles Token (XTC) - Alpha

[![Coverage Status](https://coveralls.io/repos/github/Psychedelic/dank/badge.svg?branch=main)](https://coveralls.io/github/Psychedelic/dank?branch=main)

The Cycles Token (XTC) is a cycles ledger canister that provides users with a “wrapped/tokenized” version of cycles (XTC) that can be held with just a Principal ID (no need for a Cycles Wallet), and that also includes all the same developer features and functions (calls) as the Cycles Wallet (built into the XTC token itself). 

Each Cycles Token (XTC) is backed 1-to-1 with 1 Trillion Cycles **(1 XTC = 1 Trillion Cycles)**, with cycles locked in the canister. Through the XTC canister users & developers can call/perform any traditional trade cycle actions (send, deposit, withdraw, etc.), as well as proxy canister calls funded by cycles in their XTC balance (create canister, proxy calls to canister methods, topping up cycles in calls).

- [Cycles Token (XTC) Repo & Readme](https://github.com/Psychedelic/dank/tree/main/xtc)
- [Cycles Token (XTC) Website](https://dank.ooo/xtc/) 
- [Using XTC Guide](https://docs.dank.ooo/xtc/getting-started/)

>Dank's Cycles Token (XTC) is an Alpha product and is in active development. During this testing/development period, the Dank core team will have control over the canister's upgradeability and the "stop/halt" feature to facilitate bug and security updates, prevent malicious acts, and grow the Main Dank Canister in features.
>When the project reaches a solid maturity level, it will transition towards a fully community-owned governance system.


## Wrapped ICP - WICP

Wrapped ICP (WICP) is a wrapped version of the IC's native token, ICP. Each WICP will be backed 1:1 with ICP, meaning that 1 WICP will always have the exact same value as 1 ICP. The only difference is that, unlike ICP, WICP uses the DIP20 fungible token standard that is specifically designed to allow for interoperability between dApps and other tokens.

- [Wrapped ICP Website](https://dank.ooo/wicp/) 
- [Using WICP Guide](https://docs.dank.ooo/wicp/getting-started/)
- [WICP Repo](https://github.com/psychedelic/wicp)


## Development

The canisters are written in Rust and Motoko. To develop against them requires the rust toolchain, and node to support some build scripts; please ensure these are installed.

To run the tests:

```
node build.js
cargo test
```

----

## License

Dank © Fleek LLC 2021 - [License (GPL-3.0)](https://github.com/Psychedelic/dank/blob/main/LICENSE)
