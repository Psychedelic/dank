# ICP Minting

This document tries to explain how Dank implements the functionality related to minting
ICPs and depositing the Cycles to a user's account.

The Dfinity's ICP ledger provides methods to transfer cycles across accounts, a canister
can have an account id on the ICP ledger. Users can optionally call the notify method on
the ICP ledger to notify a canister that a certain transaction has taken place.

The ICP minting process involves transferring cycles to the minting-canister, and then
notifying the minting-canister of the transaction, the canister can then exchange ICPs
to cycles using the exchange rate and send the cycles to that canister through the
subnet management API *deposit_cycles*.

So a canister can never be notified of receiving cycles from the minting canister, yet
alone passing an argument that would be the user's account id that made the transaction.

## Minting Canisters Pool

We can rent a canister to a user, the user must mint the cycles to that canister in a
set amount of time, and notify that canister, that canister will then tell dank to
increase the user's balance.
