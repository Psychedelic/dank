const setupSdr = require('../sdr/setupSdr');

const validateStats = (stats) => {
  return {
    balance_greater_then_supply: stats.balance > stats.supply,
  };
};

const checkStats = async (sdr) => {
  const stats = await sdr.stats();

  const response = validateStats(stats);

  console.log(response);

  if (!response.balance_greater_then_supply) {
    throw new Error('Stats check failed');
  }

  console.log('\n✅ Stats checks passed\n');
};

const main = async () => {
  const sdr = setupSdr();

  checkStats(sdr);
};

main();
