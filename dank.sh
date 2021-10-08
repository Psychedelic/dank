#!/bin/bash

dfxDir="/home/dan/.config/dfx"
candidDir="/home/dan/dev/psy/dank/candid"

xtcID=$(dfx canister id xtc)
piggyID=$(dfx canister id piggy-bank)
AlicePem="${dfxDir}/identity/Alice/identity.pem"
BobPem="${dfxDir}/identity/Bob/identity.pem"
CharliePem="${dfxDir}/identity/Charlie/identity.pem"
xtcCandidFile="${candidDir}/xtc.did"
piggyCandidFile="${candidDir}/piggy-bank.did"
AlicePrincipalId=$(dfx identity use Alice 2>/dev/null;dfx identity get-principal)
BobPrincipalId=$(dfx identity use Bob 2>/dev/null;dfx identity get-principal)
CharliePrincipalId=$(dfx identity use Charlie 2>/dev/null;dfx identity get-principal)
icxPrologueXtc="--candid=${xtcCandidFile}"
icxProloguePiggy="--candid=${piggyCandidFile}"

dfx identity use default 2>/dev/null

declare -A nameToPrincipal=( ["Alice"]="$AlicePrincipalId" ["Bob"]="$BobPrincipalId" ["Charlie"]="$CharliePrincipalId")
declare -A nameToPem=( ["Alice"]="$AlicePem" ["Bob"]="$BobPem" ["Charlie"]="$CharliePem")

burn(){
    fromPem="${nameToPem[$1]}"
    amount=$2
    icx --pem=$fromPem update $xtcID burn "record { canister_id= principal \"$piggyID\"; amount = $amount:nat64}" $icxPrologueXtc
}
allowance(){
    pem=$AlicePem
    from="${nameToPrincipal[$1]}"
    to="${nameToPrincipal[$2]}"
    icx --pem=$pem query $xtcID allowance "(principal \"$from\", principal \"$to\")" $icxPrologueXtc
}

decimals(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID decimals "()" $icxPrologueXtc
}

getMetadata(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID getMetadata "()" $icxPrologueXtc
}

historySize(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID historySize "()" $icxPrologueXtc
}

logo(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID logo "()" $icxPrologueXtc
}

name(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID nameErc20 "()" $icxPrologueXtc
}

nameLegacy(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID name "()" $icxPrologueXtc
}

symbol(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID symbol "()" $icxPrologueXtc
}

totalSupply(){
    pem=$AlicePem
    icx --pem=$pem query $xtcID totalSupply "()" $icxPrologueXtc
}

stats(){
	pem=$AlicePem
	icx --pem=$pem query $xtcID stats $icxPrologueXtc
}

getTransaction(){
	txId=$1
	pem=$AlicePem
	icx --pem=$pem update $xtcID getTransaction "($txId:nat)" $icxPrologueXtc
}

getTransactions(){
	txId=$1
    limit=$2
	pem=$AlicePem
	icx --pem=$pem update $xtcID getTransactions "($txId:nat, $limit:nat)" $icxPrologueXtc
}

getTransactionLegacy(){
	from=$1
	pem=$AlicePem
	icx --pem=$pem update $xtcID get_transaction "($from:nat64)" $icxPrologueXtc
}

approve(){
	pem="${nameToPem[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	icx --pem=$pem update $xtcID approve "(principal \"$to\", $amount:nat)" $icxPrologueXtc
}

transfer(){
	fromPem="${nameToPem[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	icx --pem=$fromPem update $xtcID transferErc20 "(principal \"$to\", $amount:nat)" $icxPrologueXtc
}

transferFrom(){
	from="${nameToPrincipal[$1]}"
	to="${nameToPrincipal[$2]}"
	amount=$3
	callerPem="${nameToPem[$1]}"
	if [ "$#" -eq 4 ]; then
    	callerPem="${nameToPem[$4]}"
	fi
	icx --pem=$callerPem update $xtcID transferFrom "(principal \"$from\",principal \"$to\", $amount:nat)" $icxPrologueXtc
}

balanceOf(){
	pem=$AlicePem
	account="${nameToPrincipal[$1]}"
	icx --pem=$pem query $xtcID balanceOf "(principal \"$account\")" $icxPrologueXtc
}

mint(){
	pem="${nameToPem[$1]}"
	amount="${2:-10_000_000_000_000}"
	icx --pem=$pem update $piggyID perform_mint "(record { canister= principal \"$xtcID\"; account=null; cycles=10_000_000_000_000 })" $icxProloguePiggy
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
