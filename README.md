![](https://storageapi.fleek.co/fleek-team-bucket/Dank/Banner.png)

# Dank

Dank is an infrastructure layer Open Internet Service on the Internet Computer, that provides cycle-based financial services to users and developers!

## Interacting with Dank

### On the Mainnet

Dank offers it's services on the mainnet of the Internet Computer (IC). Interacting with Dank on the mainnetis not much different than interacting with it locally.
Dank's principal ID on the mainnet is `aanaa-xaaaa-aaaah-aaeiq-cai`. You have to use this address for your calls. Let's check our
balance with this ID:

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai balance "(null)"
(0)
```

The reason that we passed `null` as a parameter to the balance method is that we want to check our own balance. If we wanted to check
another account's balance, we would've added a principal ID there. In that scenario, the command would have changes to this
(the principal ID used in the command is not real, just an example):

```bash
$ principalID="q6d6b-7t7pe-wdoiw-wjwn7-smnub-aaflq-cjd6i-luoec-gqtg3-62hiy-7qe"
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai balance "(principal \"$principalID\")"
(0)
```

There are two differences between commands that call Dank and commands that call Dank locally:

1. When we call Dank locally, the command doesn't have the `--network=ic` option.
2. When we call Dank locally, there is no need to pass the principal ID, we can just use the name `dank`.

If you keep these differences in mind, you can interact with Dank on the mainnet the same way you interact with it locally.

Withdrawing cycles (You should change the amount):

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai withdraw "(record { canister_id= principal \"some-canister's-principal-id\"; amount= 2000})"
(variant { Ok = 1 })
```

Transferring cycles to another Dank account (You should change the amount):

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai transfer "(record { to= principal \"some-account's-principal-id\"; amount= 1000 })"
(variant { Ok = 2 })
```

Depositing cycles to your Dank account (You should change AMOUNT to what you want):

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai deposit "(null)" --with-cycles AMOUNT
(variant { Ok = 3 })
```

NOTE: You can deposit cycles to another Dank account from your identity with the same `deposit` method that we used to deposit cycles to our own Dank account. For that situation, you should change `"(null)"` to a principal ID:

```bash
$ dfx canister --network=ic call aanaa-xaaaa-aaaah-aaeiq-cai deposit "(principal \"Some-Principal-ID\")" --with-cycles AMOUNT
(variant { Ok = 4 })
```

### Interacting with Dank locally

To interact with Dank and use it's methods locally, you should first clone this repository.
After that, you should start the dfx service and deploy the canisters on the IC:

```bash
git clone git@github.com:Psychedelic/dank.git
cd dank
dfx start --background --clean
dfx deploy
```

NOTE: All of the commands that are used here are put together in a shell script. You can run that shell script locally and
check the functionality of Dank without executing each command seperately. The script's location is [scripts/interactions.sh](https://github.com/Psychedelic/dank/blob/nima/scripts/interactions.sh).

Now that the canisters are deployed on the IC, we can call their methods. Some methods like `withdraw`, need other canisters.
For that reason, we have created the `piggy-bank` canister. This canister will be used to demonstrate how any other canister on
the network should interact with Dank. For now, let's just check our balance:

```bash
$ myID=$(dfx identity get-principal)
$ dfx canister call dank balance "(null)"
(0)
```

As expected, we see that our balance is initially set to zero. To play around with Dank, lets deposit some cycles to our account from
our Piggy Bank. Piggy Bank has a balance of 4TC initially. Let's deposit 4000 cycles from it using these commands:

```bash
$ dankID=$(dfx canister id dank)
$ dfx canister call piggy-bank perform_deposit "(record { canister= principal \"$dankID\"; account=null; cycles=5000 })"
(variant { Ok = 0 })
$ dfx canister call dank balance "(null)"
(5_000)
```

Oops! We made a mistake. Now we have 1000 more cycles than we needed. We should give them back to Piggy Bank. Since Piggy Bank
is a canister outside of Dank's network (like any other canister), we should withdraw one thousand cycles:

```bash
$ piggyID=$(dfx canister id piggy-bank)
$ dfx canister call dank withdraw "(record { canister_id= principal \"$piggyID\"; amount= 2000})"
(variant { Ok = 1 })
$ dfx canister call dank balance "(null)"
(4_000)
```

That's good! We just made our first call from Dank! That was calling the `withdraw` method. We only use that method when we want to
take some cycles out of our Dank account and deposit them to another canister. You might ask what if I want to transfer some cycles
to another user? Well, when we ___don't___ want to transfer cycles to a canister, we use the `transfer` method. Since there are no
other users on our local network, let's create one. To do that we need to create a new identity and transfer cycles to their principal ID:

```bash
$ dfx identity new steve || true
Creating identity: "steve".
Created identity: "steve".
$ steveID=$(dfx --identity steve identity get-principal)
$ dfx canister call dank transfer "(record { to= principal \"$steveID\"; amount= 1000 })"
(variant { Ok = 2 })
```

You might ask how does dank know from what account I'm transferring cycles? Well, that's implied in your command. If we don't add
the `--identity` flag, DFX uses your default identity and because of that, Dank also uses your default account. If you add that flag
and force DFX to use another identity, Dank also uses the account associated with that identity. For example if we wanted to make a
transfer from Steve's account, we would have had to add `--identity steve` after `dfx`:

```bash
dfx --identity steve canister call dank transfer "(record { to= principal \"some-principal-id\"; amount= 1000 })"
```

Now if we check our balance we see that it's decreased by one thousand cycles, and if we check Steve's balance we see that it is one thousand cycles:

```bash
$ dfx canister call dank balance "(null)"
(3_000)
$ dfx --identity steve canister call dank balance "(null)"
(1_000)
```

That's it! We just used `balance`, `transfer`, and `withdraw` methods. This is the basic functionality of Dank. With Dank, instead
of a wallet ID and a principal ID, you just have one principal ID that manages all of your work!
