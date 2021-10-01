const { convertFromSerializableTransaction } = require('../utils/convert');
const fs = require('fs');
const path = require('path')

const BACKUP_DIR = 'backup';

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

module.exports = readTransactionsFromBackup