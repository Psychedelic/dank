const fs = require('fs');
const path = require('path');
const setupXtc = require('../xtc/setupXtc');
const { convertToSerializableTransaction } = require('../utils/convert');
const stringify = require('../utils/stringify');

const BACKUP_DIR = 'backup';

const backupHistory = async (xtc) => {
  if (!fs.existsSync(BACKUP_DIR)) {
    fs.mkdirSync(BACKUP_DIR);
  }

  const files = fs.readdirSync(BACKUP_DIR);

  const lastTransaction = files.length - 1;

  const stats = await xtc.stats();

  const noOfTransactions = stats.history_events;

  const transactionIds = Array.from(
    Array(Number(noOfTransactions)).keys()
  ).slice(lastTransaction + 1);

  console.log(`Starting backup of Live XTC Transaction History`);
  console.log('');
  console.log(`Current number of transaction in history: ${noOfTransactions}`);
  if (lastTransaction > 0) {
    console.log(
      `Previously synced to transaction #${lastTransaction}, starting at the next transation`
    );
  }

  for (const index of transactionIds) {
    console.log(`Storing ${index} (${noOfTransactions})`);

    const result = await xtc.get_transaction(index);
    const transaction = convertToSerializableTransaction(result[0]);

    await fs.promises.writeFile(
      path.join(BACKUP_DIR, `${index}.json`),
      stringify(transaction)
    );
  }
};

const main = async () => {
  const xtc = setupXtc();

  await backupHistory(xtc);
};

main();
