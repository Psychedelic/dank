const setupXtc = require('../xtc/setupXtc');

const validateStats = (stats) => {
  return {
    balance_greater_then_supply: stats.balance > stats.supply,
  };
};

const checkStats = async (xtc) => {
  const stats = await xtc.stats();

  const response = validateStats(stats);

  console.log(response);

  if (!response.balance_greater_then_supply) {
    throw new Error('Stats check failed');
  }

  console.log('\n✅ Stats checks passed\n');
};

const main = async () => {
  const xtc = setupXtc();

  checkStats(xtc);
};

main();
