# Manual Tests

To run locally:

```shell
dfx start
```

In another tab:

```shell
dfx identity use manual-test-1
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
# Supply should be 9_998_000_000_000
# Mint count should be 1
# fee should be 2_000_000_000
# history_events should be 1

# Check your balance has the cycles
dfx canister call xtc balanceOf "principal \"$principalId\""
# shoule be (9_998_000_000_000 : nat64)

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
dfx canister call xtc balanceOf "(principal \"$secondPrincipalId\")"

# excpect returned balance as nat64: 0
```

## Transfer

Transfer to new user

```shell
# Check from balance before
dfx canister call xtc balanceOf "(principal \"$principalId\")"
dfx canister call xtc balanceOf "(principal \"$secondPrincipalId\")"

dfx canister --no-wallet call xtc transfer "(principal \"$secondPrincipalId\", 1000:nat)"

dfx canister call xtc balanceOf "(principal \"$principalId\")"
# expect balance before - transfer amount (1000) - fee (2_000_000_00) = 9_995_999_999_000

dfx canister call xtc balanceOf "(principal \"$secondPrincipalId\")"
# expect transfer amount (1000)

dfx canister call xtc stats
# expect supply is 9_996_000_000_000 (10TC - 2 fees)
# expect transfer count is 1
# expect fee to be 4_000_000_000
# expect history_events to be 2
```

Transfer back to original user

```shell
# Switch to other identity
dfx identity use manual-test-2

dfx canister call xtc balanceOf "(principal \"$principalId\")"
dfx canister call xtc balanceOf "(principal \"$secondPrincipalId\")"

dfx canister --no-wallet call xtc transfer "(principal \"$principalId\", 1000:nat)"
# Insufficient balance due to fees

dfx identity use manual-test-1
dfx canister --no-wallet call xtc transfer "(principal \"$secondPrincipalId\", 2_000_000_000:nat)"
dfx canister call xtc balanceOf "(principal \"$principalId\")"
dfx canister call xtc balanceOf "(principal \"$secondPrincipalId\")"

# Try again with fees
dfx identity use manual-test-2
dfx canister --no-wallet call xtc transfer "(principal \"$principalId\", 1000:nat)"

dfx canister call xtc balanceOf "(principal \"$secondPrincipalId\")"
# should be 0

dfx canister call xtc balanceOf "(principal \"$principalId\")"
# should be back to 9_992_000_000_000 (10TC minus fees)

dfx canister call xtc stats
# expect supply remains 9_992_000_000_000 (10TC minus fees)
# expect transfer count is 3
# expect history_events to be 4

dfx canister call xtc get_transaction "(3)"

dfx canister call xtc events "record { limit= 5: nat16 }"
# should show the 3 transactions, with the last transfer as the first entry, matching
# what you get back from get_transaction
```

Transfer 0 cycles should be accepted but charge a fee

```shell
# as identity 1
dfx identity use manual-test-1
dfx canister --no-wallet call xtc transfer "(principal \"$secondPrincipalId\", 0:nat)"
# transaction doesn't go through

dfx canister call xtc balanceOf "(principal \"$principalId\")"
# 9_990_000_000_000 - fee taken
```

## Burn

Burn to a canister:

```shell
# Check xtc balance before
dfx canister call xtc balanceOf "(principal \"$principalId\")"

# Check piggy-bank balance before
dfx canister status piggy-bank

# Burn xtc and send to piggy bank
dfx canister call xtc burn "record { canister_id= principal \"$(dfx canister id piggy-bank)\"; amount = 1000:nat64}"

# Check balance
dfx canister call xtc balanceOf "(principal \"$principalId\")"
# should be amount less than before - including the fee

# Chck piggy bank cycles balance
dfx canister status piggy-bank
# Balance should be burn amount higher

# Check transaction
dfx canister call xtc get_transaction "(5)"
dfx canister call xtc events "record { limit= 5: nat16 }"
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
dfx canister call xtc balanceOf "(principal \"$principalId\")"

dfx canister --no-wallet call xtc wallet_create_canister "(record {cycles= (1_000_000_000_000:nat64); controller= (null); })"

# Returns canister principal id

# Balance is decremented - including the fee
dfx canister call xtc balanceOf "(principal \"$principalId\")"

# check the newly created canister
dfx canister --no-wallet status rkp4c-7iaaa-aaaaa-aaaca-cai
# expect balance is as in created
# check controller is your identity (not something else like the xtc canister)

# check transaction
dfx canister call xtc get_transaction "(4)"
# expect from and canister props
dfx canister call xtc events "record { limit= 10: nat16 }"

dfx canister call xtc stats
# Supply should have dropped by creation amount
# history_events increments to 2
# fees to have an addition 2 billion
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
dfx canister call xtc balanceOf "(principal \"$principalId\")"

dfx canister call xtc wallet_call "(record { canister= principal \"$(dfx canister id piggy-bank)\"; method_name= \"whoami\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (0:nat64); })"

# check your balance after
dfx canister call xtc balanceOf "(principal \"$principalId\")"
# fee has been taken

dfx canister call xtc stats
# no increase in the no of transactions
# supply down by fee
# no change in procy call counts
```

Proxy call with enough cycles in balance

```shell
# check your balance before
dfx canister call xtc balanceOf "(principal \"$principalId\")"

dfx canister call xtc wallet_call "(record { canister= principal \"$walletId\"; method_name= \"wallet_receive\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (1000:nat64); })"

# succeeds with response

dfx canister call xtc balanceOf "(principal \"$principalId\")"
# balance decremented by 1000 + fee (2 billion)

dfx canister call xtc get_transaction "(7)"
# expect transaction with from, method_name, canister
dfx canister call xtc events "record { limit= 10: nat16 }"

# check cycles are passed along
dfx canister --no-wallet status $walletId
```

Check proxy call without enough cycles in balance:

```shell
# check your balance before
dfx canister call xtc balanceOf "(principal \"$principalId\")"

dfx canister call xtc wallet_call "(record { canister= principal \"$walletId\"; method_name= \"wallet_receive\"; args= blob \"DIDL\01nh\01\00\00\"; cycles= (100_000_000_000_000:nat64); })"

# (variant { Err = "Insufficient Balance" })
```

## Approvals

Transfer via an approval and claim.

```shell
# use identity 1s fortune to setup the allowances
dfx identity use manual-test-1

# check allowances
dfx canister --no-wallet call xtc allowance "(principal \"$principalId\", principal \"$secondPrincipalId\")"
# expect 0

dfx canister --no-wallet call xtc approve "(principal \"$secondPrincipalId\", (2000:nat))"

# Check updated allowance
dfx canister --no-wallet call xtc allowance "(principal \"$principalId\", principal \"$secondPrincipalId\")"
# expect 2000 + fee

dfx identity use manual-test-2

# attempt to transfer too much
dfx canister --no-wallet call xtc transferFrom "(principal \"$principalId\", principal \"$secondPrincipalId\", (2001:nat))"
# InsufficientAllowance

# actually transfer part of the allowance
dfx canister --no-wallet call xtc transferFrom "(principal \"$principalId\", principal \"$secondPrincipalId\", (1000:nat))"

# Check updated allowance
dfx canister --no-wallet call xtc allowance "(principal \"$principalId\", principal \"$secondPrincipalId\")"
# should be 1000

dfx canister --no-wallet call xtc transferFrom "(principal \"$principalId\", principal \"$secondPrincipalId\", (1000:nat))"
# InsufficientAllowance

# Check updated allowance
dfx canister --no-wallet call xtc allowance "(principal \"$principalId\", principal \"$secondPrincipalId\")"
# should be 1000

dfx canister call xtc balanceOf "(principal \"$secondPrincipalId\")"
# should be 2_000

dfx canister call xtc get_transaction "(10)"
# Should be of the TransferFrom type
# to/from/caller

dfx canister call xtc events "record { limit= 5: nat16 }"
```

## Transaction

Lookup transaction of each type:

```shell
dfx canister call xtc get_transaction "(5)"
```

Lookup transaction out of bounds:

```shell
dfx canister call xtc get_transaction "(100)"
# (null)
```

Lookup events

```shell
dfx canister call xtc events "record { limit= 1: nat16 }"
dfx canister call xtc events "record { offset= (opt 5); limit= 3: nat16 }"
dfx canister call rno2w-sqaaa-aaaaa-aaacq-cai events "(record { offset=120:nat64; limit= 100: nat16 })"
```
