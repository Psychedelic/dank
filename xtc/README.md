![](https://storageapi.fleek.co/fleek-team-bucket/xtc-trx.png)

# Cycles Token (XTC)

The Cycles Token (XTC) is Dank's first product. A cycles canister that provides users with a “wrapped/tokenized” version of cycles (XTC) **that can be held with just a Principal ID** (no need for a Cycles Wallet). The Cycles Token (XTC) was built following a Principal-ID centric token standard [(Repository)](https://github.com/Psychedelic/standards).

**The Cycles Token (XTC) also has built-in developer features and functionality (like Cycles Wallet have)**, built into the XTC token itself so that it can be used to **create and manage canisters through proxy calls, or develop in DFX** funding the cycles fees from your Cycles Token balance.

Each XTC token represents and is backed **1-to-1 with 1 Trillion Cycles (1 XTC = 1 Trillion Cycles)**  that they can hold, utilize, pay for computation, and trade with just like with any other token, tied to their Principal ID (and only requiring a Principal ID).

- Cycles Token (XTC) Canister ID: ```aanaa-xaaaa-aaaah-aaeiq-cai```
- [View Canister on IC Rocks](https://ic.rocks/principal/aanaa-xaaaa-aaaah-aaeiq-cai)
- [Visit our Website for more Details](https://dank.ooo/xtc/)
- [XTC Documentation - Overview](https://docs.dank.ooo/xtc/overview/)

## Interacting with Cycles Token (XTC) - On Mainnet (DFX)

Cycles Token (XTC) offers its services on the mainnet of the Internet Computer (IC). Interacting with XTC on the mainnet is not much different than interacting with it locally.

XTC's Token Canister ID on the mainnet is `aanaa-xaaaa-aaaah-aaeiq-cai`. **You have to use this address for your calls**.

> Here you will find a **sample of some of the basic methods XTC offers**, for the **full interaction and methods guide visit our documentation page:

- [XTC's Complete Documentation](https://docs.dank.ooo/)


###  Check Your Balance - balanceOf

Returns the balance of user `who`.

```bash
dfx canister --network=ic --no-wallet call --query aanaa-xaaaa-aaaah-aaeiq-cai balanceOf "(principal \"who-account-principal\")"
```

### Deposit cycles to mint an XTC balance - Mint

You can get your first Cycles Token (XTC) balance by either depositing cycles to the XTC Token Canister to mint them (see below), or getting a one-time redeem of 100$ worth of cycles from DFINITY's [Cycles Faucet tool](https://faucet.dfinity.org/), selecting the option to redeem them as Dank's Cycles Token (XTC)! 

(**If you used the faucet already**, and chose the Cycles Wallet option but want to migrate to Cycles Tokens (XTC) [see this example](#sending-your-faucet-cycles-wallet-balance-to-cycles-token-xtc).)

#### Depositing from a personal Cycles Wallet

You can deposit cycles into the XTC canister from a personal cycles wallet directly. The cycles are locked in the XTC canister to "mint" your 1-1 Cycles Token (XTC), tied to your Principal ID. To send cycles from a personal cycle wallet it must be deployed to a public subnet on mainnet, it must have cycles and be set as the wallet against dfx. In the following `mint` command the AMOUNT to deposit is in cycles (You should change the amount). 

```bash
$ dfx canister --network=ic --wallet=$(dfx identity --network=ic get-wallet) call --with-cycles AMOUNT aanaa-xaaaa-aaaah-aaeiq-cai mint "(principal \"$(dfx identity get-principal)\",0:nat)"
```

NOTE: You can deposit cycles to another XTC balance from your identity with the same `mint` method that we used to deposit cycles to our own XTC balance. For that situation, you should change the argument to a principal ID:

```bash
$ dfx canister --network=ic --wallet=$(dfx identity --network=ic get-wallet) call --with-cycles AMOUNT aanaa-xaaaa-aaaah-aaeiq-cai mint "(principal \"Some-Principal-ID\",0:nat")"
```

Note: This command should not require the `--wallet` flag, but we need the `--wallet` to make `--with-cycles` work. This is a known DFX bug.

#### Sending your Faucet Cycles Wallet Balance to Cycles Token (XTC)

Did you use DFINITY's Cycles Faucet tool, but selected the Cycles Wallet to receive your redeem? Want to move that balance to Cycles Token (XTC?). **If you have ```bc``` installed** you can do this quick command.

(Make sure you set your cycles wallet as your default dfx wallet first)

```bash
dfx canister --network=ic --wallet=$(dfx identity --network=ic get-wallet) call --with-cycles $(echo "$(dfx wallet --network=ic balance | cut -d' ' -f1)-10000000000" | bc) aanaa-xaaaa-aaaah-aaeiq-cai mint "(principal \"$(dfx identity get-principal)\")"
```

---


### Withdrawing cycles to a Canister - Burn

Unwraps Cycles Token (XTC) into raw Cycles to send them to a Canister ID. (You should change the amount)

```bash
$ dfx canister --network=ic --no-wallet call aanaa-xaaaa-aaaah-aaeiq-cai burn "(record { canister_id= principal \"some-canister's-principal-id\"; amount= (2000:nat64)})"
(variant { Ok = 1 })
```

---

### Transfer XTC to another XTC Balance - transferErc20
Send Cycles Token (XTC) to a Principal ID, balances change internally on the XTC ledger. (You should change the amount).

Transfers `value` amount of tokens to user `to`, returns a `TxReceipt` which contains the transaction index or an error message.


```bash
dfx canister --network=ic --no-wallet call aanaa-xaaaa-aaaah-aaeiq-cai transferErc20 "(principal \"some-account's-principal-id\", 1000:nat)"
```

---

###  Transfer XTC on Another User's Behalf - transferFrom

Transfers `value` amount of tokens from user `from` to user `to`, this method allows canister smart contracts to transfer tokens on your behalf, it returns a `TxReceipt` which contains the transaction index or an error message.

```bash
dfx canister --network=ic --no-wallet call aanaa-xaaaa-aaaah-aaeiq-cai transferFrom "(principal \"from-account-principal\",principal \"to-account-principal\", 1000:nat)"
```

---

###  Set an Allowance to Another Identity - approve

You can set an allowance using this method, giving a third-party access to a specific number of tokens they can withdraw from your balance if they want.

An allowance permits the `spender` to withdraw tokens from your account, up to the `value` amount. If it is called again it overwrites the current allowance with `value`. There is no upper limit for value.

```bash
dfx canister --network=ic --no-wallet call aanaa-xaaaa-aaaah-aaeiq-cai approve "(principal \"third-party-account-principal\", 1000:nat)"
```

### 🔋 Create and Manage Canisters

You can create canisters using your Cycles Token (XTC) balance. This is, however, a low level api, if you want to deploy your canister using your XTC balance see [Using dfx deploy with Cycles Token](#using-dfx-deploy-with-cycles-token-xtc).

Using `wallet_create_canister` method, you can create a canister and set the controller of
the canister to a principal ID you want. If you leave the controller to be `null`, you will be automatically selected as the controller of the newly created canister. Using the `cycles` parameter, it is possible to deposit cycles to your new canister from your XTC balance.

```bash
$ dfx canister --network=ic --no-wallet call aanaa-xaaaa-aaaah-aaeiq-cai wallet_create_canister "(record {cycles= (AMOUNT:nat64); controller= (null); })"
(
  variant {
    17_724 = record { 1_313_628_723 = principal "CREATED_CANISTER_ID" }
  },
)
```

To check the status of the created canister run the dfx canister `status` command with the returned `CREATED_CANISTER_ID`:

```bash
dfx canister --network=ic --no-wallet status CREATED_CANISTER_ID
```

---

### Proxy canister calls with XTC:

XTC allows you to proxy all of your `dfx` calls through it so your Cycles Token (XTC) balance is used to fund the operations (the XTC canister unwraps them to raw cycles). To use this feature, you should use the `wallet_call` method. This method accepts four arguments:

  - canister: principal -> Your target canister
  - method_name: text -> The method you want to call from that canister
  - args: blob -> The arguments you should pass to for the call
  - cycles: nat64 -> The amount of cycles you want to pass

Let's proxy a call to the Piggy Bank canister's `whoami` method (an example canister we deployed to show an example of a proxy call!). We expect this method to return our XTC balance's ID:

```bash
dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai wallet_call "(record { canister= principal \"dmj37-5iaaa-aaaad-qakya-cai\"; method_name= \"whoami\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (0:nat64); })"
```


## ✅ Set Cycles Token (XTC) as your default wallet in dfx:

The dfx cli tool provides helper functions during development that consumes cycles from your wallet. You can set the XTC canister to be used for these functions.

```bash
dfx identity --network=ic set-wallet aanaa-xaaaa-aaaah-aaeiq-cai --force
```

### Using dfx deploy with Cycles Token (XTC):

The `dfx deploy` command shows an error when deploying within a dfx project when the cycle wallet is set as the `XTC` canister. The `deploy` command successfully creates the canister, but fails when installing the wasm code (this is due to dfx assuming the controller of the new canister is the cycle wallet, not the dfx identity).

To deploy a projects canisters, instead of `dfx deploy --network=ic` separate the canister and install commands:

```bash
dfx canister --network=ic create --all
dfx deploy --network=ic --no-wallet
```

As an example of setting up and deploying a new project once dfx has been installed:

```bash
# Set XTC as the dfx cycle token
dfx identity --network=ic set-wallet aanaa-xaaaa-aaaah-aaeiq-cai --force

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

### Interacting with XTC locally (For Testing Purposes)

There is one difference between interacting with XTC locally and interacting with XTC on the mainnet. That difference is that
when you are trying to call a method on the mainnet, you have to add the `--network=ic` flag. If you keep this one difference in mind, you can interact with XTC on the mainnet the same way you interact with it locally.

To interact with XTC and use it's methods locally, you should first clone this repository.
After that, you should start the dfx service and deploy the canisters on the IC:

```bash
git clone git@github.com:Psychedelic/dank.git
cd dank
dfx start --background --clean
dfx deploy
```

NOTE: All of the commands that are used here are put together in a shell script. You can run that shell script locally and
check the functionality of XTC without executing each command seperately. The script's location is [scripts/interactions.sh](https://github.com/Psychedelic/dank/blob/nima/scripts/interactions.sh).

Now that the canisters are deployed on the IC, we can call their methods. Some methods like `burn`, need other canisters.
For that reason, we have created the `piggy-bank` canister. This canister will be used to demonstrate how any other canister on
the network should interact with Cycles Token (XTC) canister. For now, let's just check our balance:

```bash
$ myID=$(dfx identity get-principal)
$ dfx canister call xtc balance "(null)"
(0)
```

As expected, we see that our balance is initially set to zero. To play around with XTC, **lets deposit some cycles to our account from
our Piggy Bank**. Piggy Bank has a balance of 4TC initially. Let's deposit 4000 cycles from it using these commands:

```bash
$ xtcID=$(dfx canister id xtc)
$ dfx canister call piggy-bank perform_mint "(record { canister= principal \"$xtcID\"; account=null; cycles=5000 })"
(variant { Ok = 0 })
$ dfx canister call xtc balance "(null)"
(5_000)
```

Oops! We made a mistake. Now we have 1000 more cycles than we needed. We should give them back to Piggy Bank. Since Piggy Bank
is a canister outside of XTC (like any other canister), we should **withdraw one thousand cycles**:

```bash
$ piggyID=$(dfx canister id piggy-bank)
$ dfx canister call xtc burn "(record { canister_id= principal \"$piggyID\"; amount= 2000})"
(variant { Ok = 1 })
$ dfx canister call xtc balance "(null)"
(4_000)
```

That's good! We just made our first call from XTC! That was calling the `withdraw` method. We only use that method when we want to
take some cycles out of our XTC account and deposit them to another canister. When that happens, the XTC Canister "unwraps" or "unlocks" the cycles stored (that represent your Cycles Token balance), and sends the raw cycles to the canister.

You might ask what if I want to transfer some cycles to another user? **(sending Cycles Token, XTC, to another Principal ID)** Well, when we ___don't___ want to transfer raw cycles to a canister, we use the `transfer` method, which simply reduces your balance on the XTC canister and increases the balance of the destination Principal ID (internally on the ledger as well). Since there are no other users on our local network, let's create one. To do that we need to create a new identity and transfer cycles to their principal ID:

```bash
$ dfx identity new steve || true
Creating identity: "steve".
Created identity: "steve".
$ steveID=$(dfx --identity steve identity get-principal)
$ dfx canister call xtc transfer "(record { to= principal \"$steveID\"; amount= 1000 })"
(variant { Ok = 2 })
```

You might ask how does the XTC canister know from what account I'm transferring cycles? Well, that's implied in your command. If we don't add
the `--identity` flag, DFX uses your default identity and because of that, XTC also uses your default account. If you add that flag
and force DFX to use another identity, XTC also uses the account associated with that identity. For example if we wanted to make a
transfer from Steve's account, we would have had to add `--identity steve` after `dfx`:

```bash
dfx --identity steve canister call xtc transfer "(record { to= principal \"some-principal-id\"; amount= 1000 })"
```

Now if we check our balance we see that it's decreased by one thousand cycles, and if we check Steve's balance we see that it is one thousand cycles:

```bash
$ dfx canister call xtc balance "(null)"
(3_000)
$ dfx --identity steve canister call xtc balance "(null)"
(1_000)
```

That's it! We just used `balance`, `transfer`, and `withdraw` methods. This is the basic functionality of XTC. With XTC, instead
of a wallet ID and a principal ID, you just have one principal ID that manages all of your work!

# Transaction History (Events Log)

To achieve scalability the XTC canister is developed to utilize multiple canisters to store its transactions
logs when needed. Although it is using multiple canisters, the `get_transaction(txId)` method on the main
canister is able to return all the transactions in the log.

There is also the secondary method called `events` which is used to paginate the entire events log on multiple
canisters. This method takes an optional `offset` and a `limit` and returns the transactions starting from the
given offset and moves back in the history and returns up to `limit` number of transactions.

A cursor is returned from the `events` method which contains the page you have requested for and also the next offset
along with the next canister id you have to make the request to. When there is no more data to read `null` is returned
for `next_canister_id`.