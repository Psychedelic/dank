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
echo "${action} Deploying XDR and Piggy Bank on IC"
echo
dfx deploy

# Step 3. We get Piggy Bank's and our balance.
echo
echo "${action} Getting the balances of Piggy Bank and our XDR account"
echo
piggyBalance=$(dfx canister call piggy-bank balance)
xdrBalance=$(dfx canister call xdr balance "(null)")
echo "Piggy Bank's balance: $piggyBalance"
echo "Our XDR account's balance: $xdrBalance"

# Step 4. We deposit some cycles to our XDR account from Piggy-Bank.
echo
echo "${action} Depositing 5000 cycles to our XDR account from Piggy Bank"
echo
xdrID=$(dfx canister id xdr)
dfx canister call piggy-bank perform_mint "(record { canister= principal \"$xdrID\"; account=null; cycles=5000 })"

echo
piggyBalance=$(dfx canister call piggy-bank balance)
xdrBalance=$(dfx canister call xdr balance "(null)")
echo "Piggy Bank's new balance: $piggyBalance"
echo "Our XDR account's balance: $xdrBalance"

# Step 5. We withdraw some cycles from XDR.
echo
echo "${action} Withdrawing 2000 cycles from XDR to Piggy Bank"
echo
piggyID=$(dfx canister id piggy-bank)
dfx canister call xdr burn "(record { canister_id= principal \"$piggyID\"; amount= 2000})"

echo
piggyBalance=$(dfx canister call piggy-bank balance)
xdrBalance=$(dfx canister call xdr balance "(null)")
echo "Piggy Bank's new balance: $piggyBalance"
echo "Our XDR account's balance: $xdrBalance"

# Step 6. We create a new identity and transfer some cycles to it.
echo
echo "${action} Creating a new identity named steve and transferring 1000 cycles to it."
echo
dfx identity new steve || true
steveID=$(dfx --identity steve identity get-principal)
dfx canister call xdr transfer "(record { to= principal \"$steveID\"; amount= 1000 })"

echo
steveBalance=$(dfx --identity steve canister call xdr balance "(null)")
xdrBalance=$(dfx canister call xdr balance "(null)")
echo "Steve's new balance: $steveBalance"
echo "Our XDR account's balance: $xdrBalance"


# Now that we're done let's stop the service.
echo
echo "${action} Stopping the service"
echo
dfx stop
