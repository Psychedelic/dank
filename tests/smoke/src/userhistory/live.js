const readTransactionsFromBackup = require('../utils/readTransactionsFromBackup');

const filterByUser = (userId) => ({ kind }) => {
  if (kind.Mint) {
    return kind.Mint.to.toString() === userId
  } else if (kind.Transfer) {
    return kind.Transfer.to.toString() === userId || kind.Transfer.from.toString() === userId
  } else if (kind.Burn) {
    return kind.Burn.to.toString() === userId || kind.Burn.from.toString() === userId
  } else if (kind.CanisterCreated) {
    return kind.CanisterCreated.from.toString() === userId
  } else if (kind.CanisterCalled) {
    return kind.CanisterCalled.from.toString() === userId
  }

  return true
}

const main = async () => {
  const transactions = await readTransactionsFromBackup()

  const userId = process.argv[2]

  if (!userId) {
    console.log(`Usage: yarn userhistory:live 5jehy-ivhe5-cxbh3-dkc2n-4d3cb-4u34q-3uo77-5z4r2-kdivi-lgsn2-uae`)
    process.exit(1)
  }

  console.log('')
  console.log('User History')
  console.log('============')
  console.log('')
  console.log(`User Id: ${userId}`)
  console.log('')

  for (const transaction of transactions.filter(filterByUser(userId))) {
    console.log(transaction)
  }
};

main();
