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