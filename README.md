# Dank

Dank is an infrastructure layer Open Internet Service on the Internet Computer, that provides cycle-based financial services to users and developers!

## Interacting with Dank

To interact with Dank and use it's methods, you should first clone this repository.
After that, you should start dfx and deploy the dank canister on the IC:

```bash
git clone git@github.com:Psychedelic/dank.git
cd dank
dfx start --background --clean
dfx deploy
```

Now that the canister is deployed on the IC, we can call it's methods. Some methods like `transfer`, need other canisters.
For now, lets just check our balance:

```bash
$ myID=$(dfx identity get-principal)
$ dfx canister call dank balance "(principal \"$myID\")"
(0)
```
