# Manual Tests

To run locally:

```shell
dfx start
```

In another tab:

```shell
dfx deploy --with-cycles 48000000000000 xtc
dfx deploy --with-cycles 12000000000000 piggy-bank
```

## Run Tests

To setup use local variables:

```shell
xtcID=$(dfx canister id xtc)  
walletId=$(dfx identity get-wallet)
principalId=$(dfx identity get-principal)

dfx identity use manual-test-2
secondPrincipalId=$(dfx identity get-principal)
dfx identity use manual-test-1
```

### Mint to your account

```shell
# Mint into XTC based on your account, via piggy-bank
dfx canister call piggy-bank perform_mint "(record { canister= principal \"$xtcID\"; account=null; cycles=10_000_000_000_000 })"

# Check the stats
dfx canister call xtc stats
# Supply should be 10_000_000_000_000
# Mint count should be 1
# history_events should be 1

# Check your balance has the cycles
dfx canister call xtc balance "(null)"
# shoule be (10_000_000_000_000 : nat64)

# Check the transaction history
dfx canister call xtc get_transaction "(0)"

# Something like:
# (
#   opt record {
#     fee = 0 : nat64;
#     kind = variant {
#       Mint = record {
#         to = principal "a5ygl-e4uzd-md3mh-mkbkb-7jsep-443v4-jsgv6-g3sz3-xhbpk-si7c7-kae";
#       }
#     };
#     cycles = 10_000_000_000_000 : nat64;
#     timestamp = 1_629_228_297_715 : nat64;
#   },
# )
#
# Check events
dfx canister call xtc events "record { limit= 5: nat16 }"

# Should see 1 record and the correct recort
```

## Check balances

For your account:

```
dfx canister call xtc balance "(null)"
# Verify number is what you expect
```

For a non-existant account

```shell
dfx canister call xtc balance "(opt principal \"$secondPrincipalId\")"

# excpect returned balance as nat64
```

## Transfer

Transfer to new user

```shell
# Check from balance before
dfx canister call xtc balance "(opt principal \"$principalId\")"
dfx canister call xtc balance "(opt principal \"$secondPrincipalId\")"

dfx canister --no-wallet call xtc transfer "(record { to= principal \"$secondPrincipalId\"; amount= (1000:nat64) })"

dfx canister call xtc balance "(opt principal \"$principalId\")"
# expect balance before - transfer amount (1000)

dfx canister call xtc balance "(opt principal \"$secondPrincipalId\")"
# expect transfer amount (1000)

dfx canister call xtc stats
# expect supply remains 10TC
# expect transfer count is 1
# expect history_events to be 2
```

Transfer back to original user

```shell
# Switch to other identity 
dfx identity use manual-test-2

dfx canister call xtc balance "(opt principal \"$principalId\")"
dfx canister call xtc balance "(opt principal \"$secondPrincipalId\")"

dfx canister --no-wallet call xtc transfer "(record { to= principal \"$principalId\"; amount= (1000:nat64) })"

dfx canister call xtc balance "(opt principal \"$secondPrincipalId\")"
# should be 0

dfx canister call xtc balance "(opt principal \"$principalId\")"
# should be back to 10TC

dfx canister call xtc stats
# expect supply remains 10TC
# expect transfer count is 2
# expect history_events to be 3

dfx canister call xtc get_transaction "(2)"

dfx canister call xtc events "record { limit= 5: nat16 }"
# should show the 3 transactions, with the last transfer as the first entry, matching
# what you get back from get_transaction
```

Transfer 0 cycles should be rejected

```shell
# as identity 1 
dfx canister --no-wallet call xtc transfer "(record { to= principal \"$secondPrincipalId\"; amount= (0:nat64) })"

# expect error -  Transaction is expected to have a non-zero amount
```

## Burn

Burn to a canister:

```shell
# Check xtc balance before
dfx canister call xtc balance "(null)"

# Check piggy-bank balance before
dfx canister status piggy-bank

# Burn xtc and send to piggy bank
dfx canister call xtc burn "record { canister_id= principal \"$(dfx canister id piggy-bank)\"; amount = 1000:nat64}"

# Check balance
dfx canister call xtc balance "(null)"
# should be amount less than before

# Chck piggy bank cycles balance
dfx canister status piggy-bank
# Balance should be burn amount higher

# Check transaction
dfx canister call xtc get_transaction "(3)"
dfx canister call xtc events "record { from= (opt 0); limit= 5: nat16 }"
```

Burn cycles you don't have:

```shell
dfx canister call xtc burn "record { canister_id= principal \"$(dfx canister id piggy-bank)\"; amount = 1_000_000_000_000_000:nat64}"

# insufficient balance error
```

Burn to non-existant canister:aanaa-xaaaa-aaaah-aaeiq-cai

```shell
dfx canister call xtc burn "record { canister_id= principal \"aanaa-xaaaa-aaaah-aaeiq-cai\"; amount = 1_000:nat64}"

# error invalid token contract
```

Burn to user principal id

```shell
dfx canister call xtc burn "record { canister_id= principal \"a5ygl-e4uzd-md3mh-mkbkb-7jsep-443v4-jsgv6-g3sz3-xhbpk-si7c7-kae\"; amount = 1_000:nat64}"

# error invalid token contract
```

## Create Canister

Create a canister with 1TC

```shell
# Check personal balance before
dfx canister call xtc balance "(null)"

dfx canister --no-wallet call xtc wallet_create_canister "(record {cycles= (1_000_000_000_000:nat64); controller= (null); })"

# Returns canister principal id

# Balance is decremented
dfx canister call xtc balance "(null)"

# check the newly created canister
dfx canister --no-wallet status rkp4c-7iaaa-aaaaa-aaaca-cai
# expect balance is as in created
# check controller is your identity (not something else like the xtc canister)

# check transaction
dfx canister call xtc get_transaction "(4)"
# expect from and canister props
dfx canister call xtc events "record { from= (opt 0); limit= 10: nat16 }"

dfx canister call xtc stats
# Supply should have dropped by creation amount
# history_events increments to 5
# canister_created_count is 1
```

Create a canister with more cycles than is in balance

```shell
dfx canister --no-wallet call xtc wallet_create_canister "(record {cycles= (100_000_000_000_000:nat64); controller= (null); })"

# expect insufficient balance
```

Check that managment functions are limited to creating identity

```shell
# Switch to different identity

dfx canister --no-wallet status rkp4c-7iaaa-aaaaa-aaaca-cai
# Expect error - Requested canister rejected the message
```

## Proxy call

Proxy call with 0 additional cycles:

```shell
# check your balance before
dfx canister call xtc balance "(null)"

dfx canister call xtc wallet_call "(record { canister= principal \"$(dfx canister id piggy-bank)\"; method_name= \"whoami\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (0:nat64); })"

# check your balance after
dfx canister call xtc balance "(null)"
# no change in balance

dfx canister call xtc stats
# no increase in the no of transactions
# no change in supply
# no change in procy call counts
```

Proxy call with enough cycles in balance

```shell
# check your balance before
dfx canister call xtc balance "(null)"

dfx canister call xtc wallet_call "(record { canister= principal \"$walletId\"; method_name= \"wallet_receive\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (1000:nat64); })"

# succeeds with response

dfx canister call xtc balance "(null)"
# balance decremented by 1000

dfx canister call xtc get_transaction "(5)"
# expect transaction with from, method_name, canister
dfx canister call xtc events "record { from= (opt 0); limit= 10: nat16 }"

# check cycles are passed along
dfx canister --no-wallet status $walletId
```

Check proxy call without enough cycles in balance:

```shell
# check your balance before
dfx canister call xtc balance "(null)"

dfx canister call xtc wallet_call "(record { canister= principal \"$walletId\"; method_name= \"wallet_receive\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (100_000_000_000_000:nat64); })"

# (variant { Err = "Insufficient Balance" })
```

## Transaction

Lookup transaction of each type:

```shell
dfx canister call xtc get_transaction "(5)"
```

Lookup transaction out of bounds:

```shell
dfx canister call xtc get_transaction "(100)"
```

Lookup events

```shell
dfx canister call xtc 


dfx canister call xtc events "record { from= (opt 4); limit= 1: nat16 }"
```
