function stringify(value) {
  if (value !== undefined) {
    return JSON.stringify(
      value,
      (_, v) => (typeof v === 'bigint' ? `${v}n` : v),
      2
    );
  }
}

module.exports = stringify;
