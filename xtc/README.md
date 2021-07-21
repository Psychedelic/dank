![](https://storageapi.fleek.co/fleek-team-bucket/xtc-trx.png)

# Cycles Token (XTC)

The Cycles Token (XTC) is Dank's first product. A cycles canister that provides users with a “wrapped/tokenized” version of cycles (XTC) **that can be held with just a Principal ID** (no need for a Cycles Wallet). The Cycles Token (XTC) was built following a Principal-ID centric token standard [(Repository)](https://github.com/Psychedelic/standards).

**The Cycles Token (XTC) also has built-in developer features and functionality (like Cycles Wallet have)**, built into the XTC token itself so that it can be used to **create and manage canisters through proxy calls, or develop in DFX** funding the cycles fees from your Cycles Token balance.

Each XTC token represents and is backed **1-to-1 with 1 Trillion Cycles (1 XTC = 1 Trillion Cycles)**  that they can hold, utilize, pay for computation, and trade with just like with any other token, tied to their Principal ID (and only requiring a Principal ID).

## Interacting with Cycles Token (XTC)

- Cycles Token (XTC) Canister ID: ```aanaa-xaaaa-aaaah-aaeiq-cai```
- [View Canister on IC Rocks](https://ic.rocks/principal/aanaa-xaaaa-aaaah-aaeiq-cai)
- [Visit our Website for more Details](https://dank.ooo/xtc/)
- [XTC Documentation - Overview](https://docs.dank.ooo/xtc/overview/)

### On the Mainnet - DFX

Cycles Token (XTC) offers its services on the mainnet of the Internet Computer (IC). Interacting with XTC on the mainnet is not much different than interacting with it locally.
XTC's Token Canister ID on the mainnet is `aanaa-xaaaa-aaaah-aaeiq-cai`. You have to use either this address for your calls or just simply use `xtc`.

**Checking your balance with this ID on XTC:**

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai balance "(null)"
(0)
$ dfx canister --network=ic --no-wallet call xtc balance "(null)"
(0)
```

The reason that we passed `null` as a parameter to the balance method is that we want to check our own balance. If we wanted to check
another account's balance, **we would've added a principal ID there**. In that scenario, the command would have changes to this
(the principal ID used in the command is not real, just an example):

```bash
$ principalID="q6d6b-7t7pe-wdoiw-wjwn7-smnub-aaflq-cjd6i-luoec-gqtg3-62hiy-7qe"
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai balance "(principal \"$principalID\")"
(0)
```

- **Withdrawing Cycles to Canister** -(Unwrapping Cycles Token (XTC) into Cycles and sending to Canister ID):

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai burn "(record { canister_id= principal \"some-canister's-principal-id\"; amount= 2000})"
(variant { Ok = 1 })
```

- **Transferring cycles to another XTC balance/Principal ID** (Sending XTC to a Principal ID, balances change internally on the XTC ledger):**

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai transfer "(record { to= principal \"some-account's-principal-id\"; amount= 1000 })"
(variant { Ok = 2 })
```

- **Depositing cycles to your Cycles Token (XTC) balance** (Deposit cycles, which are locked, "minting" your 1-1 XTC balance):

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai mint "(null)" --with-cycles AMOUNT
(variant { Ok = 3 })
```

NOTE: You can deposit cycles to another XTC balance from your identity with the same `mint` method that we used to deposit cycles to our own XTC balance. For that situation, you should change `"(null)"` to a principal ID:

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai mint "(principal \"Some-Principal-ID\")" --with-cycles AMOUNT
(variant { Ok = 4 })
```

- **Creating canisters using XTC**:

You can create canisters using your XTC balance. Using `wallet_create_canister` method, you can create a canister and set the controller of
the canister to a principal ID you want. If you leave the controller to be `null`, you will be automatically selected as the controller of the newly created canister. Using the `cycles` parameter, it is possible to deposit cycles to your new canister from your XTC balance.

```bash
dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai wallet_create_canister "(record (cycles: (AMOUNT:nat64); controller: (\"null\"); ))"

```

- **Proxy calls with XTC**:

XTC allows you to proxy all of your `dfx` calls through it so your XTC balance is used. To use this feature, you should use the `wallet_call` method. This method accepts four arguments:

  - canister: principal -> Your target canister
  - method_name: text -> The method you want to call from that canister
  - args: blob -> The arguments you should pass to for the call
  - cycles: nat64 -> The amount of cycles you want to pass

Let's proxy a call to the Piggy Bank canister's `whoami` method (an example canister we deployed to show an example of a proxy call!). We expect this method to return our XTC balance's ID:

```bash
dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai wallet_call "(record { canister= principal \"dmj37-5iaaa-aaaad-qakya-cai\"; method_name= \"whoami\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (0:nat64); })"
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
