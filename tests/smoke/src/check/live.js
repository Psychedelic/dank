const setupXdr = require('../xdr/setupXdr');

const validateStats = (stats) => {
  return {
    balance_greater_then_supply: stats.balance > stats.supply,
  };
};

const checkStats = async (xdr) => {
  const stats = await xdr.stats();

  const response = validateStats(stats);

  console.log(response);

  if (!response.balance_greater_then_supply) {
    throw new Error('Stats check failed');
  }

  console.log('\n✅ Stats checks passed\n');
};

const main = async () => {
  const xdr = setupXdr();

  checkStats(xdr);
};

main();
