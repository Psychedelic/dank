#!/bin/sh

# Step 0. We change the directory to root and stop dfx (if running).
echo
echo [ACTION] Stopping DFX
echo
cd ..
dfx stop

# Step 1. We start the dfx service
echo
echo [ACTION] Starting DFX
echo
dfx start --background --clean

# Step 2. Let's deploy our canisters on IC.
echo
echo [ACTION] Deploying Dank and Piggy Bank on IC
echo
dfx deploy

# Step 3. We get Piggy Bank's and Dank's balance.
echo
echo [ACTION] Getting the balances of our canisters
echo
piggyBalance=$(dfx canister call piggy-bank balance)
dankBalance=$(dfx canister call dank balance "(null)")
echo Piggy Bank\'s balance: "$piggyBalance"
echo Dank\'s balance: "$dankBalance"

# Step 4. We deposit some cycles to Dank from Piggy-Bank.
echo
echo [ACTION] Depositing 5000 cycles to Dank from Piggy Bank
echo
dankID=$(dfx canister id dank)
dfx canister call piggy-bank perform_deposit "(record { canister= principal \"$dankID\"; account=null; cycles=5000 })"

echo
piggyBalance=$(dfx canister call piggy-bank balance)
dankBalance=$(dfx canister call dank balance "(null)")
echo Piggy Bank\'s new balance: "$piggyBalance"
echo Dank\'s new balance: "$dankBalance"

# Step 5. We withdraw some cycles from Dank.
echo
echo [ACTION] Withdrawing 2000 cycles from Dank to Piggy Bank
echo
piggyID=$(dfx canister id piggy-bank)
dfx canister call dank withdraw "(record { canister_id= principal \"$piggyID\"; amount= 2000})"

echo
piggyBalance=$(dfx canister call piggy-bank balance)
dankBalance=$(dfx canister call dank balance "(null)")
echo Piggy Bank\'s new balance: "$piggyBalance"
echo Dank\'s new balance: "$dankBalance"

# Step 6. We transfer some of the cycles back to piggy-bank using the transfer method from Dank
echo
echo [ACTION] Transfering 1000 cycles back to Piggy Bank
echo
dfx canister call dank transfer "(record { to= principal \"$piggyID\"; amount= 1000 })"

echo
piggyBalance=$(dfx canister call piggy-bank balance)
dankBalance=$(dfx canister call dank balance "(null)")
echo Piggy Bank\'s new balance: "$piggyBalance"
echo Dank\'s new balance: "$dankBalance"


# Now that we're done let's stop the service.
echo
echo [ACTION] Stopping the service
echo
dfx stop