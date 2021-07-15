# Dank

Dank is an infrastructure layer Open Internet Service on the Internet Computer, that provides cycle-based financial services to users and developers!

## Interacting with Dank

To interact with Dank and use it's methods, you should first clone this repository.
After that, you should start dfx and deploy our canisters on the IC:

```bash
git clone git@github.com:Psychedelic/dank.git
cd dank
dfx start --background --clean
dfx deploy
```

Now that the canister is deployed on the IC, we can call it's methods. Some methods like `withdraw`, need other canisters.
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

Now if we check our balance we see that it's decreased by one thousand cycles, and if we check Steve's balance we see that it is one thousand cycles:

```bash
$ dfx canister call dank balance "(null)"
(3_000)
$ dfx --identity steve canister call dank balance "(null)"
(1_000)
```

That's it! We just used `balance`, `transfer`, and `withdraw` methods. This is the basic functionality of Dank. With Dank, instead
of a wallet ID and a principal ID, you just have one principal ID that manages all of your work!
