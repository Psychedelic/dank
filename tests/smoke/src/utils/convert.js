const { Principal } = require('@dfinity/principal');

function convertStringToBigInt(text) {
  if (!text.endsWith('n')) {
    throw new Error('Not a big int string');
  }

  return BigInt(text.substring(0, text.length - 1));
}

const convertToSerializableTransaction = (transactionResponse) => {
  if (!transactionResponse.kind) {
    throw new Error('Not a transaction, no kind property');
  }

  if (transactionResponse.kind.Mint) {
    transactionResponse.kind.Mint.to =
      transactionResponse.kind.Mint.to.toText();

    return transactionResponse;
  }

  if (transactionResponse.kind.Burn) {
    transactionResponse.kind.Burn.to =
      transactionResponse.kind.Burn.to.toText();
    transactionResponse.kind.Burn.from =
      transactionResponse.kind.Burn.from.toText();

    return transactionResponse;
  }

  if (transactionResponse.kind.Transfer) {
    transactionResponse.kind.Transfer.to =
      transactionResponse.kind.Transfer.to.toText();
    transactionResponse.kind.Transfer.from =
      transactionResponse.kind.Transfer.from.toText();

    return transactionResponse;
  }

  if (transactionResponse.kind.CanisterCreated) {
    transactionResponse.kind.CanisterCreated.canister =
      transactionResponse.kind.CanisterCreated.canister.toText();
    transactionResponse.kind.CanisterCreated.from =
      transactionResponse.kind.CanisterCreated.from.toText();

    return transactionResponse;
  }

  if (transactionResponse.kind.CanisterCalled) {
    transactionResponse.kind.CanisterCalled.canister =
      transactionResponse.kind.CanisterCalled.canister.toText();
    transactionResponse.kind.CanisterCalled.from =
      transactionResponse.kind.CanisterCalled.from.toText();

    return transactionResponse;
  }

  if (transactionResponse.kind.Approve) {
    transactionResponse.kind.Approve.to =
      transactionResponse.kind.Approve.to.toText();
    transactionResponse.kind.Approve.from =
      transactionResponse.kind.Approve.from.toText();

    return transactionResponse;
  }
  
  if (transactionResponse.kind.TransferFrom) {
    transactionResponse.kind.TransferFrom.to =
      transactionResponse.kind.TransferFrom.to.toText();
    transactionResponse.kind.TransferFrom.from =
      transactionResponse.kind.TransferFrom.from.toText();
    transactionResponse.kind.TransferFrom.caller =
      transactionResponse.kind.TransferFrom.caller.toText();

    return transactionResponse;
  }

  throw new Error(
    `Unknown transaction kind - ${JSON.stringify(transactionResponse.kind)}`
  );
};

const convertFromSerializableTransaction = (serializableTransaction) => {
  if (!serializableTransaction.kind) {
    throw new Error('Not a transaction, no kind property');
  }

  serializableTransaction.fee = convertStringToBigInt(
    serializableTransaction.fee
  );
  serializableTransaction.cycles = convertStringToBigInt(
    serializableTransaction.cycles
  );
  serializableTransaction.timestamp = convertStringToBigInt(
    serializableTransaction.timestamp
  );

  if (serializableTransaction.kind.Mint) {
    serializableTransaction.kind.Mint.to = Principal.fromText(
      serializableTransaction.kind.Mint.to
    );

    return serializableTransaction;
  }

  if (serializableTransaction.kind.Burn) {
    serializableTransaction.kind.Burn.to = Principal.fromText(
      serializableTransaction.kind.Burn.to
    );
    serializableTransaction.kind.Burn.from = Principal.fromText(
      serializableTransaction.kind.Burn.from
    );

    return serializableTransaction;
  }

  if (serializableTransaction.kind.Transfer) {
    serializableTransaction.kind.Transfer.to = Principal.fromText(
      serializableTransaction.kind.Transfer.to
    );
    serializableTransaction.kind.Transfer.from = Principal.fromText(
      serializableTransaction.kind.Transfer.from
    );

    return serializableTransaction;
  }

  if (serializableTransaction.kind.CanisterCreated) {
    serializableTransaction.kind.CanisterCreated.canister = Principal.fromText(
      serializableTransaction.kind.CanisterCreated.canister
    );
    serializableTransaction.kind.CanisterCreated.from = Principal.fromText(
      serializableTransaction.kind.CanisterCreated.from
    );

    return serializableTransaction;
  }

  if (serializableTransaction.kind.CanisterCalled) {
    serializableTransaction.kind.CanisterCalled.canister = Principal.fromText(
      serializableTransaction.kind.CanisterCalled.canister
    );
    serializableTransaction.kind.CanisterCalled.from = Principal.fromText(
      serializableTransaction.kind.CanisterCalled.from
    );

    return serializableTransaction;
  }

  if (serializableTransaction.kind.Approve) {
    serializableTransaction.kind.Approve.to = Principal.fromText(
      serializableTransaction.kind.Approve.to
    );
    serializableTransaction.kind.Approve.from = Principal.fromText(
      serializableTransaction.kind.Approve.from
    );

    return serializableTransaction;
  }

  if (serializableTransaction.kind.TransferFrom) {
    serializableTransaction.kind.TransferFrom.to = Principal.fromText(
      serializableTransaction.kind.TransferFrom.to
    );
    serializableTransaction.kind.TransferFrom.from = Principal.fromText(
      serializableTransaction.kind.TransferFrom.from
    );
    serializableTransaction.kind.TransferFrom.caller = Principal.fromText(
      serializableTransaction.kind.TransferFrom.caller
    );

    return serializableTransaction;
  }

  throw new Error(
    `Unknown transaction kind - ${JSON.stringify(serializableTransaction.kind)}`
  );
};

module.exports = {
  convertToSerializableTransaction,
  convertFromSerializableTransaction,
};
