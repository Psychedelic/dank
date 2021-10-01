# Smoke Tests

Scripts to support checking and verifying a mainnet deploy of the XTC canister.

## Install

The scripts are written in `node` and rely on `yarn`, within the `./tests/smoke` directory install the dependencies by running:

```shell
yarn
```

## Tests

### Stats Check

To confirm the stats response of live is within paramters run:

```shell
yarn check:live
```

### Backup

To backup the transaction history as json files to a local directory run:

```shell
yarn backup:live
```

The script will sync the difference each time it is run. By default it backs up to `./backup`


### Verify

This requires a backup to have been run. Verify uses the backup to reconstruct the expected ledger and then checks each entry in the expected ledger against the live ledger.

To run:

```shell
yarn verify:live
```

### Whales

To determine the largest holders of XTC

To run:

```shell
yarn whales:live
```

### User History

Show a users transaction history

To run:

```shell
yarn userhistory:live 5jehy-ivhe5-cxbh3-dkc2n-4d3cb-4u34q-3uo77-5z4r2-kdivi-lgsn2-uae
```