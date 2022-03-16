#!/bin/sh

redBG=$(tput setab 1)
reset=$(tput sgr0)
action=$(echo "${redBG}[ACTION]${reset}")

# Step 0. We change the directory to root and stop dfx (if running).
echo
echo "${action} Stopping DFX"
echo
cd ..
dfx stop

# Step 1. We start the dfx service
echo
echo "${action} Starting DFX"
echo
dfx start --background --clean

# Step 2. Let's deploy our canisters on IC.
echo
echo "${action} Deploying SDR and Piggy Bank on IC"
echo
dfx deploy

# Step 3. We get Piggy Bank's and our balance.
echo
echo "${action} Getting the balances of Piggy Bank and our SDR account"
echo
piggyBalance=$(dfx canister call piggy-bank balance)
sdrBalance=$(dfx canister call sdr balance "(null)")
echo "Piggy Bank's balance: $piggyBalance"
echo "Our SDR account's balance: $sdrBalance"

# Step 4. We deposit some cycles to our SDR account from Piggy-Bank.
echo
echo "${action} Depositing 5000 cycles to our SDR account from Piggy Bank"
echo
sdrID=$(dfx canister id sdr)
dfx canister call piggy-bank perform_mint "(record { canister= principal \"$sdrID\"; account=null; cycles=5000 })"

echo
piggyBalance=$(dfx canister call piggy-bank balance)
sdrBalance=$(dfx canister call sdr balance "(null)")
echo "Piggy Bank's new balance: $piggyBalance"
echo "Our SDR account's balance: $sdrBalance"

# Step 5. We withdraw some cycles from SDR.
echo
echo "${action} Withdrawing 2000 cycles from SDR to Piggy Bank"
echo
piggyID=$(dfx canister id piggy-bank)
dfx canister call sdr burn "(record { canister_id= principal \"$piggyID\"; amount= 2000})"

echo
piggyBalance=$(dfx canister call piggy-bank balance)
sdrBalance=$(dfx canister call sdr balance "(null)")
echo "Piggy Bank's new balance: $piggyBalance"
echo "Our SDR account's balance: $sdrBalance"

# Step 6. We create a new identity and transfer some cycles to it.
echo
echo "${action} Creating a new identity named steve and transferring 1000 cycles to it."
echo
dfx identity new steve || true
steveID=$(dfx --identity steve identity get-principal)
dfx canister call sdr transfer "(record { to= principal \"$steveID\"; amount= 1000 })"

echo
steveBalance=$(dfx --identity steve canister call sdr balance "(null)")
sdrBalance=$(dfx canister call sdr balance "(null)")
echo "Steve's new balance: $steveBalance"
echo "Our SDR account's balance: $sdrBalance"


# Now that we're done let's stop the service.
echo
echo "${action} Stopping the service"
echo
dfx stop
