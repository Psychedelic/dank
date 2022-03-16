#!/bin/bash

dfxDir="/home/dan/.config/dfx"
candidDir="/home/dan/dev/psy/dank/candid"

xdrID=$(dfx canister id xdr)
piggyID=$(dfx canister id piggy-bank)
AlicePem="${dfxDir}/identity/Alice/identity.pem"
BobPem="${dfxDir}/identity/Bob/identity.pem"
CharliePem="${dfxDir}/identity/Charlie/identity.pem"
xdrCandidFile="${candidDir}/xdr.did"
piggyCandidFile="${candidDir}/piggy-bank.did"
AlicePrincipalId=$(dfx identity use Alice 2>/dev/null;dfx identity get-principal)
BobPrincipalId=$(dfx identity use Bob 2>/dev/null;dfx identity get-principal)
CharliePrincipalId=$(dfx identity use Charlie 2>/dev/null;dfx identity get-principal)
icxPrologueXdr="--candid=${xdrCandidFile}"
icxProloguePiggy="--candid=${piggyCandidFile}"

dfx identity use default 2>/dev/null

declare -A nameToPrincipal=( ["Alice"]="$AlicePrincipalId" ["Bob"]="$BobPrincipalId" ["Charlie"]="$CharliePrincipalId")
declare -A nameToPem=( ["Alice"]="$AlicePem" ["Bob"]="$BobPem" ["Charlie"]="$CharliePem")

burn(){
    fromPem="${nameToPem[$1]}"
    amount=$2
    icx --pem=$fromPem update $xdrID burn "record { canister_id= principal \"$piggyID\"; amount = $amount:nat64}" $icxPrologueXdr
}
allowance(){
    pem=$AlicePem
    from="${nameToPrincipal[$1]}"
    to="${nameToPrincipal[$2]}"
    icx --pem=$pem query $xdrID allowance "(principal \"$from\", principal \"$to\")" $icxPrologueXdr
}

decimals(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID decimals "()" $icxPrologueXdr
}

getMetadata(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID getMetadata "()" $icxPrologueXdr
}

historySize(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID historySize "()" $icxPrologueXdr
}

logo(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID logo "()" $icxPrologueXdr
}

name(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID nameErc20 "()" $icxPrologueXdr
}

nameLegacy(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID name "()" $icxPrologueXdr
}

symbol(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID symbol "()" $icxPrologueXdr
}

totalSupply(){
    pem=$AlicePem
    icx --pem=$pem query $xdrID totalSupply "()" $icxPrologueXdr
}

stats(){
	pem=$AlicePem
	icx --pem=$pem query $xdrID stats $icxPrologueXdr
}

getTransaction(){
	txId=$1
	pem=$AlicePem
	icx --pem=$pem update $xdrID getTransaction "($txId:nat)" $icxPrologueXdr
}

getTransactions(){
	txId=$1
    limit=$2
	pem=$AlicePem
	icx --pem=$pem update $xdrID getTransactions "($txId:nat, $limit:nat)" $icxPrologueXdr
}

getTransactionLegacy(){
	from=$1
	pem=$AlicePem
	icx --pem=$pem update $xdrID get_transaction "($from:nat64)" $icxPrologueXdr
}

approve(){
	pem="${nameToPem[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	icx --pem=$pem update $xdrID approve "(principal \"$to\", $amount:nat)" $icxPrologueXdr
}

transfer(){
	fromPem="${nameToPem[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	icx --pem=$fromPem update $xdrID transferErc20 "(principal \"$to\", $amount:nat)" $icxPrologueXdr
}

transferFrom(){
	from="${nameToPrincipal[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	callerPem="${nameToPem[$1]}"
	if [ "$#" -eq 4 ]; then
    	callerPem="${nameToPem[$4]}"
	fi
	icx --pem=$callerPem update $xdrID transferFrom "(principal \"$from\",principal \"$to\", $amount:nat)" $icxPrologueXdr
}

balanceOf(){
	pem=$AlicePem
	account="${nameToPrincipal[$1]}"
	icx --pem=$pem query $xdrID balanceOf "(principal \"$account\")" $icxPrologueXdr
}

mint(){
	pem="${nameToPem[$1]}"
	amount="${2:-10_000_000_000_000}"
	icx --pem=$pem update $piggyID perform_mint "(record { canister= principal \"$xdrID\"; account=null; cycles=10_000_000_000_000 })" $icxProloguePiggy
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
