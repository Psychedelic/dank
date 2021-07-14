#!/bin/sh

# Step 0. We change the directory to root and stop dfx (if running).
cd ..
dfx stop

# Step 1. We start the dfx service
echo
echo == Starting DFX
echo
dfx start --background --clean

# Step 2. Let's deploy our canisters on IC.
echo
echo == Deploying Dank and Piggy Bank on IC
echo
dfx deploy

# Step 3. We get

# Now that we're done let's stop the service.
echo
echo == Stopping the service
echo
dfx stop