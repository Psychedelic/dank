const fs = require('fs');
const path = require('path');
const { Principal } = require('@dfinity/principal');
const setupXtc = require('../xtc/setupXtc');
const { convertFromSerializableTransaction } = require('../utils/convert');
const stringify = require('../utils/stringify');

const BACKUP_DIR = 'backup';
const SNAPSHOT_DIR = 'snapshot';
const A_TRILLION = BigInt(1000000000000);

const readTransactionsFromBackup = async () => {
  if (!fs.existsSync(BACKUP_DIR)) {
    throw new Error('No backup dir exists');
  }

  const transactions = [];

  let tranCount = 0;

  while (fs.existsSync(path.join(BACKUP_DIR, `${tranCount}.json`))) {
    // console.log(`Reading ${tranCount}.json`)

    const file = path.join(BACKUP_DIR, `${tranCount}.json`);
    const transactionText = JSON.parse(fs.readFileSync(file));

    const transaction = convertFromSerializableTransaction(transactionText);

    transactions.push(transaction);

    tranCount++;
  }

  return transactions;
};

const applyTransaction = (ledger, transaction) => {
  if (transaction.kind.Mint) {
    const to = transaction.kind.Mint.to.toText();
    const cycles = transaction.cycles;

    let entry = ledger[to] || BigInt(0);
    entry = entry + cycles;
    ledger[to] = entry;
  }

  if (transaction.kind.Transfer) {
    const to = transaction.kind.Transfer.to.toText();
    const from = transaction.kind.Transfer.from.toText();
    const cycles = transaction.cycles;

    // increment to
    let entry = ledger[to] || BigInt(0);
    entry = entry + cycles;
    ledger[to] = entry;

    // decrement from
    let fromEntry = ledger[from];
    fromEntry = fromEntry - cycles;
    ledger[from] = fromEntry;
  }

  if (transaction.kind.Burn) {
    const from = transaction.kind.Burn.from.toText();
    const cycles = transaction.cycles;

    // decrement from
    let fromEntry = ledger[from];
    fromEntry = fromEntry - cycles;
    ledger[from] = fromEntry;
  }

  if (transaction.kind.CanisterCreated) {
    const from = transaction.kind.CanisterCreated.from.toText();

    const cycles = transaction.cycles;

    // decrement from
    let fromEntry = ledger[from];
    fromEntry = fromEntry - cycles;
    ledger[from] = fromEntry;
  }

  if (transaction.kind.CanisterCalled) {
    const from = transaction.kind.CanisterCalled.from.toText();
    const cycles = transaction.cycles;

    // decrement from
    let fromEntry = ledger[from];
    fromEntry = fromEntry - cycles;
    ledger[from] = fromEntry;
  }

  return ledger;
};

const saveBalances = async (xtc, principalIds = []) => {
  if (!fs.existsSync(SNAPSHOT_DIR)) {
    fs.mkdirSync(SNAPSHOT_DIR);
  }

  const balancesFile = path.join(SNAPSHOT_DIR, `balances.json`);

  let balances = {};
  if (fs.existsSync(balancesFile)) {
    balances = JSON.parse(fs.readFileSync(balancesFile));
  }

  count = 0;

  for (const principalId of principalIds) {
    const principal = Principal.fromText(principalId);
    const result = await xtc.balance([principal]);

    balances[principalId] = result;

    await fs.promises.writeFile(balancesFile, stringify(balances));

    count++;
    console.log(`Checking balance for ${principalId} (${count}/${principalIds.length})`);
  }
};

const verifyBalances = async (ledger) => {
  const balances = JSON.parse(
    await fs.promises.readFile(path.join(SNAPSHOT_DIR, `balances.json`))
  );

  const unexplainedDiffs = [];
  for (const principalId of Object.keys(ledger)) {
    const balanceString = balances[principalId];

    if (!balanceString) {
      unexplainedDiffs.push(principalId);
      continue;
    }

    const balance = BigInt(
      balanceString.substring(0, balanceString.length - 1)
    );

    const ledgerBalance = ledger[principalId];

    if (balance === ledgerBalance) {
      // console.log(`✅ ${principalId}`)
    } else {
      unexplainedDiffs.push(principalId);
      console.log(
        `❌ ${principalId}`,
        balance,
        ledgerBalance,
        ledgerBalance - balance
      );
    }
  }

  console.log('');
  console.log(`Differences ${unexplainedDiffs.length}`);

  return unexplainedDiffs;
};

const buildLedgerFromBackup = async () => {
  const transactions = await readTransactionsFromBackup();
  const ledger = {};

  for (const transaction of transactions) {
    applyTransaction(ledger, transaction);
  }

  return ledger;
};

const main = async () => {
  const xtc = setupXtc();

  const ledger = await buildLedgerFromBackup();

  const whales = Object.entries(ledger).filter(([principalId, balance]) => balance > BigInt(100000000000000n))

  console.log('')
  console.log('XTC Whales - any account with more than 100 XTC')
  console.log('-----------------------------------------------')
  console.log('')
  console.log(`There are currently ${whales.length} whales`)
  console.log('')

  for (const [principalId, balance] of whales.sort((a,b) => (a[1] < b[1]) ? 1 : ((a[1] > b[1]) ? -1 : 0))) {
    console.log(`${balance / A_TRILLION} XTC - ${principalId}`)
  }

  console.log()
};

main();
