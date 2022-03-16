# Special Drawing Rights (SDR) Token
[![Documentation](https://img.shields.io/badge/Documentation-2CA5E0?color=blue)](https://docs.earthwallet.io/sdr)
[![Canister](https://img.shields.io/badge/Canister-Deployed-2CA5E0?color=success)](https://ic.rocks/principal/qlttm-2yaaa-aaaak-qafvq-cai)
[![Discord](https://img.shields.io/badge/Discord-%237289DA.svg?style=flat&logo=discord&logoColor=white)](https://discord.gg/aemgEpMye3)
[![Twitter](https://img.shields.io/badge/Twitter-%231DA1F2.svg?style=flat&logo=Twitter&logoColor=white)](https://twitter.com/earthwallet)
![License](https://img.shields.io/badge/Code-MIT%20Licence-blueviolet)

The Internet Computer utilizes a stablecoin known as Special Drawing Rights ([SDR](https://www.imf.org/en/Topics/special-drawing-right)) to pay for compute, storage, and bandwidth costs of decentralized applications running on the network. 1 SDR represents 1 Trillion cycles on the Internet Computer.

 may vary by usage, but can be viewed here [Cycle costs on the Internet Computer](https://smartcontracts.org/docs/developers-guide/computation-and-storage-costs.html)

 The price of 1 SDR = $1.42 USD as of writing, but this changes based on the value of the currencies it represents. Specifically it is made of the US dollar, Euro, Chinese Yuan, Japanese Yen, and the British Pound. An up to date overview of the SDR basket of currencies and value can be found [here](https://www.imf.org/external/np/fin/data/rms_sdrv.aspx). These numbers are automatically updated on the Internet Computer Protocol to maintain the 1 SDR <> 1T cycle peg.

## SDR Token & Cycles

Cycles on the Internet Computer must be held in a cycles wallet, which requires command line developer interfaces to access. However, the SDR token is implemented as a tokenized version which provides users the ability to hold it with just a principal ID. The SDR token was derived from the work of [Psycadelic DAO](https://github.com/Psychedelic/standards).

The SDR token aims to provide easy to use developer and end user experience for internet computer developers to create, maintain, and fund canisters without the need for complex tooling. For example, end users may need the ability to fund their social media profiles with cycles, without having to know how command line tools work.

## Interacting with SDR - Mainnet

SDR's Token Canister ID on the mainnet is `qlttm-2yaaa-aaaak-qafvq-cai`. **You have to use this address for your calls**.

For a full overview please read the full [SDR Documentation](https://docs.earthwallet.io/sdr).

SDR Token offers its services on the mainnet of the Internet Computer (IC). Interacting with SDR on the mainnet is not much different than interacting with it locally.

Once you transfer SDR, or cycles into [Earth Wallet](https://earthdao.co) you can easily send, receive, mint, burn cycles directly all in within a single easy to use interface. Creating, managing, and funding canisters can also easily be done using the canister manager app.

### Deposit cycles to mint an SDR balance - Mint

You can get your first SDR Token balance by either depositing cycles to the SDR Token Canister to mint them (see below), or using a swap protocol (coming soon) to convert ICP or any other IC token to SDR.

#### Depositing from a personal Cycles Wallet

You can deposit cycles into the SDR canister from a personal cycles wallet directly. The cycles are locked in the SDR canister to "mint" your 1-1 SDR Token (SDR), tied to your Principal ID. To send cycles from a personal cycle wallet it must be deployed to a public subnet on mainnet, it must have cycles and be set as the wallet against dfx. In the following `mint` command the AMOUNT to deposit is in cycles (You should change the amount).

```bash
$ dfx canister --network=ic --wallet=$(dfx identity --network=ic get-wallet) call --with-cycles AMOUNT qlttm-2yaaa-aaaak-qafvq-cai mint "(principal \"$(dfx identity get-principal)\",0:nat)"
```

NOTE: You can deposit cycles to another SDR balance from your identity with the same `mint` method that we used to deposit cycles to our own SDR balance. For that situation, you should change the argument to a principal ID:

```bash
$ dfx canister --network=ic --wallet=$(dfx identity --network=ic get-wallet) call --with-cycles AMOUNT qlttm-2yaaa-aaaak-qafvq-cai mint "(principal \"Some-Principal-ID\",0:nat")"
```

Note: This command should not require the `--wallet` flag, but we need the `--wallet` to make `--with-cycles` work. This is a known DFX bug.

---

### How to Deploy SDR Token:

The `dfx deploy` command shows an error when deploying within a dfx project when the cycle wallet is set as the `SDR` canister. The `deploy` command successfully creates the canister, but fails when installing the wasm code (this is due to dfx assuming the controller of the new canister is the cycle wallet, not the dfx identity).

To deploy a projects canisters, instead of `dfx deploy --network=ic` separate the canister and install commands:

```bash
dfx canister --network=ic create --all
dfx deploy --network=ic --no-wallet
```

As an example of setting up and deploying a new project once dfx has been installed:

```bash
# Set SDR as the dfx cycle token
dfx identity --network=ic set-wallet qlttm-2yaaa-aaaak-qafvq-cai --force

# Create a new dfx project
dfx new example

# move into the project directory
cd example

# install the node.js dependencies
npm install

# Create the empty canisters on mainnet
dfx canister --network=ic create --all

# Install the code into the empty canisters on mainnet
dfx deploy --network=ic --no-wallet
```



----

### Interacting with SDR locally (For Testing Purposes)

There is one difference between interacting with SDR locally and interacting with SDR on the mainnet. That difference is that
when you are trying to call a method on the mainnet, you have to add the `--network=ic` flag. If you keep this one difference in mind, you can interact with SDR on the mainnet the same way you interact with it locally.

To interact with SDR and use it's methods locally, you should first clone this repository.
After that, you should start the dfx service and deploy the canisters on the IC:

```bash
git clone git@github.com:earthdao/SDR.git
cd SDR
dfx start --background --clean
dfx deploy
```

NOTE: All of the commands that are used here are put together in a shell script. You can run that shell script locally and
check the functionality of SDR without executing each command seperately. The script's location is [scripts/interactions.sh](https://github.com/earthdao/SDR/blob/nima/scripts/interactions.sh).

Now that the canisters are deployed on the IC, we can call their methods. Some methods like `burn`, need other canisters.
For that reason, we have created the `piggy-bank` canister. This canister will be used to demonstrate how any other canister on
the network should interact with SDR Token  canister. For now, let's just check our balance:

```bash
$ myID=$(dfx identity get-principal)
$ dfx canister call sdr balance "(null)"
(0)
```

As expected, we see that our balance is initially set to zero. To play around with SDR, **lets deposit some cycles to our account from
our Piggy Bank**. Piggy Bank has a balance of 4TC initially. Let's deposit 4000 cycles from it using these commands:

```bash
$ sdrID=$(dfx canister id sdr)
$ dfx canister call piggy-bank perform_mint "(record { canister= principal \"$sdrID\"; account=null; cycles=5000 })"
(variant { Ok = 0 })
$ dfx canister call sdr balance "(null)"
(5_000)
```

Oops! We made a mistake. Now we have 1000 more cycles than we needed. We should give them back to Piggy Bank. Since Piggy Bank
is a canister outside of SDR (like any other canister), we should **withdraw one thousand cycles**:

```bash
$ piggyID=$(dfx canister id piggy-bank)
$ dfx canister call sdr burn "(record { canister_id= principal \"$piggyID\"; amount= 2000})"
(variant { Ok = 1 })
$ dfx canister call sdr balance "(null)"
(4_000)
```

That's good! We just made our first call from SDR! That was calling the `withdraw` method. We only use that method when we want to
take some cycles out of our SDR account and deposit them to another canister. When that happens, the SDR Canister "unwraps" or "unlocks" the cycles stored (that represent your Cycles Token balance), and sends the raw cycles to the canister.

You might ask what if I want to transfer some cycles to another user? **(sending SDR, to another Principal ID)** Well, when we ___don't___ want to transfer raw cycles to a canister, we use the `transfer` method, which simply reduces your balance on the SDR canister and increases the balance of the destination Principal ID (internally on the ledger as well). Since there are no other users on our local network, let's create one. To do that we need to create a new identity and transfer cycles to their principal ID:

```bash
$ dfx identity new steve || true
Creating identity: "steve".
Created identity: "steve".
$ steveID=$(dfx --identity steve identity get-principal)
$ dfx canister call sdr transfer "(record { to= principal \"$steveID\"; amount= 1000 })"
(variant { Ok = 2 })
```

You might ask how does the SDR canister know from what account I'm transferring cycles? Well, that's implied in your command. If we don't add
the `--identity` flag, DFX uses your default identity and because of that, SDR also uses your default account. If you add that flag
and force DFX to use another identity, SDR also uses the account associated with that identity. For example if we wanted to make a
transfer from Steve's account, we would have had to add `--identity steve` after `dfx`:

```bash
dfx --identity steve canister call sdr transfer "(record { to= principal \"some-principal-id\"; amount= 1000 })"
```

Now if we check our balance we see that it's decreased by one thousand cycles, and if we check Steve's balance we see that it is one thousand cycles:

```bash
$ dfx canister call sdr balance "(null)"
(3_000)
$ dfx --identity steve canister call sdr balance "(null)"
(1_000)
```

That's it! We just used `balance`, `transfer`, and `withdraw` methods. This is the basic functionality of SDR. With SDR, instead
of a wallet ID and a principal ID, you just have one principal ID that manages all of your work!

# Transaction History (Events Log)

To achieve scalability the SDR canister is developed to utilize multiple canisters to store its transactions
logs when needed. Although it is using multiple canisters, the `get_transaction(txId)` method on the main
canister is able to return all the transactions in the log.

There is also the secondary method called `events` which is used to paginate the entire events log on multiple
canisters. This method takes an optional `offset` and a `limit` and returns the transactions starting from the
given offset and moves back in the history and returns up to `limit` number of transactions.

A cursor is returned from the `events` method which contains the page you have requested for and also the next offset
along with the next canister id you have to make the request to. When there is no more data to read `null` is returned
for `next_canister_id`.

## Development

The canisters are written in Rust and Motoko. To develop against them requires the rust toolchain, and node to support some build scripts; please ensure these are installed.

To build locally

```
dfx start --background --clean(in one tab)
dfx deploy sdr
dfx deploy piggy-bank
```

To run the tests:

```
node buildsdr.js
cargo test
```
