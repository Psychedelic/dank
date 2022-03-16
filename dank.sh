#!/bin/bash

dfxDir="/home/dan/.config/dfx"
candidDir="/home/dan/dev/psy/dank/candid"

sdrID=$(dfx canister id sdr)
piggyID=$(dfx canister id piggy-bank)
AlicePem="${dfxDir}/identity/Alice/identity.pem"
BobPem="${dfxDir}/identity/Bob/identity.pem"
CharliePem="${dfxDir}/identity/Charlie/identity.pem"
sdrCandidFile="${candidDir}/sdr.did"
piggyCandidFile="${candidDir}/piggy-bank.did"
AlicePrincipalId=$(dfx identity use Alice 2>/dev/null;dfx identity get-principal)
BobPrincipalId=$(dfx identity use Bob 2>/dev/null;dfx identity get-principal)
CharliePrincipalId=$(dfx identity use Charlie 2>/dev/null;dfx identity get-principal)
icxPrologueSdr="--candid=${sdrCandidFile}"
icxProloguePiggy="--candid=${piggyCandidFile}"

dfx identity use default 2>/dev/null

declare -A nameToPrincipal=( ["Alice"]="$AlicePrincipalId" ["Bob"]="$BobPrincipalId" ["Charlie"]="$CharliePrincipalId")
declare -A nameToPem=( ["Alice"]="$AlicePem" ["Bob"]="$BobPem" ["Charlie"]="$CharliePem")

burn(){
    fromPem="${nameToPem[$1]}"
    amount=$2
    icx --pem=$fromPem update $sdrID burn "record { canister_id= principal \"$piggyID\"; amount = $amount:nat64}" $icxPrologueSdr
}
allowance(){
    pem=$AlicePem
    from="${nameToPrincipal[$1]}"
    to="${nameToPrincipal[$2]}"
    icx --pem=$pem query $sdrID allowance "(principal \"$from\", principal \"$to\")" $icxPrologueSdr
}

decimals(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID decimals "()" $icxPrologueSdr
}

getMetadata(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID getMetadata "()" $icxPrologueSdr
}

historySize(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID historySize "()" $icxPrologueSdr
}

logo(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID logo "()" $icxPrologueSdr
}

name(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID nameErc20 "()" $icxPrologueSdr
}

nameLegacy(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID name "()" $icxPrologueSdr
}

symbol(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID symbol "()" $icxPrologueSdr
}

totalSupply(){
    pem=$AlicePem
    icx --pem=$pem query $sdrID totalSupply "()" $icxPrologueSdr
}

stats(){
	pem=$AlicePem
	icx --pem=$pem query $sdrID stats $icxPrologueSdr
}

getTransaction(){
	txId=$1
	pem=$AlicePem
	icx --pem=$pem update $sdrID getTransaction "($txId:nat)" $icxPrologueSdr
}

getTransactions(){
	txId=$1
    limit=$2
	pem=$AlicePem
	icx --pem=$pem update $sdrID getTransactions "($txId:nat, $limit:nat)" $icxPrologueSdr
}

getTransactionLegacy(){
	from=$1
	pem=$AlicePem
	icx --pem=$pem update $sdrID get_transaction "($from:nat64)" $icxPrologueSdr
}

approve(){
	pem="${nameToPem[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	icx --pem=$pem update $sdrID approve "(principal \"$to\", $amount:nat)" $icxPrologueSdr
}

transfer(){
	fromPem="${nameToPem[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	icx --pem=$fromPem update $sdrID transferErc20 "(principal \"$to\", $amount:nat)" $icxPrologueSdr
}

transferFrom(){
	from="${nameToPrincipal[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	callerPem="${nameToPem[$1]}"
	if [ "$#" -eq 4 ]; then
    	callerPem="${nameToPem[$4]}"
	fi
	icx --pem=$callerPem update $sdrID transferFrom "(principal \"$from\",principal \"$to\", $amount:nat)" $icxPrologueSdr
}

balanceOf(){
	pem=$AlicePem
	account="${nameToPrincipal[$1]}"
	icx --pem=$pem query $sdrID balanceOf "(principal \"$account\")" $icxPrologueSdr
}

mint(){
	pem="${nameToPem[$1]}"
	amount="${2:-10_000_000_000_000}"
	icx --pem=$pem update $piggyID perform_mint "(record { canister= principal \"$sdrID\"; account=null; cycles=10_000_000_000_000 })" $icxProloguePiggy
}

topup(){
    dfx canister deposit-cycles 10000000000000 ryjl3-tyaaa-aaaaa-aaaba-cai
}

setup(){
    topup
    mint Alice
    transfer Alice Bob 10
    approve Alice Bob 1000
}
