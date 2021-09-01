# Setup a manual-test-1 and manual-test-2
# dfx identity new manual-test-1
# dfx identity new manual-test-2
#
# - in root
# dfx deploy piggy-bank
# dfx deploy --with-cycles 96000000000000 xtc 
# ./tests/smoke/lots_of_transactions.sh


xtcID=$(dfx canister id xtc)  

dfx identity use manual-test-1
manualTest1=$(dfx identity get-principal)
dfx identity use manual-test-2
manualTest2=$(dfx identity get-principal)

dfx identity use manual-test-1
dfx canister call piggy-bank perform_mint "(record { canister= principal \"$xtcID\"; account=null; cycles=1_000_000_000_000 })"
dfx canister call piggy-bank perform_mint "(record { canister= principal \"$xtcID\"; account= opt principal \"$manualTest2\"; cycles=1_000_000_000_000 })"

count=300
for index in $(seq $count); do
    if [ `expr $index % 2` -eq 0 ]
    then
       dfx identity use manual-test-1
       dfx canister --no-wallet call xtc transfer "(record { to= principal \"$manualTest2\"; amount= ($index:nat64) })"
    else
       dfx identity use manual-test-2
       dfx canister --no-wallet call xtc transfer "(record { to= principal \"$manualTest1\"; amount= ($index:nat64) })"
    fi
done