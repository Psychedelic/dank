#!/bin/bash

set -e

DANK_CANISTER_DIR=./dank

echo
echo "== Dank canister serve"
echo

(
  cd "$DANK_CANISTER_DIR"

  dfx start --background --clean

  dfx deploy
)

echo
echo "== Dank canister serve"
echo "== Completed!"
echo