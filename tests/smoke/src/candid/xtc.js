const idlFactory = ({ IDL }) => {
  const TransactionId = IDL.Nat64;
  const BurnError = IDL.Variant({
    InsufficientBalance: IDL.Null,
    InvalidTokenContract: IDL.Null,
    NotSufficientLiquidity: IDL.Null,
  });
  const BurnResult = IDL.Variant({ Ok: TransactionId, Err: BurnError });
  const EventDetail = IDL.Variant({
    ChargingStationDeployed: IDL.Record({ canister: IDL.Principal }),
    Burn: IDL.Record({ to: IDL.Principal, from: IDL.Principal }),
    Mint: IDL.Record({ to: IDL.Principal }),
    CanisterCreated: IDL.Record({ canister: IDL.Principal }),
    CanisterCalled: IDL.Record({
      method_name: IDL.Text,
      canister: IDL.Principal,
    }),
    Transfer: IDL.Record({ to: IDL.Principal, from: IDL.Principal }),
  });
  const Event = IDL.Record({
    fee: IDL.Nat64,
    kind: EventDetail,
    cycles: IDL.Nat64,
    timestamp: IDL.Nat64,
  });
  const EventsConnection = IDL.Record({
    data: IDL.Vec(Event),
    next_canister_id: IDL.Opt(IDL.Principal),
  });
  const TokenMetaData = IDL.Record({
    features: IDL.Vec(IDL.Text),
    name: IDL.Text,
    decimal: IDL.Nat8,
    symbol: IDL.Text,
  });
  const MintError = IDL.Variant({ NotSufficientLiquidity: IDL.Null });
  const MintResult = IDL.Variant({ Ok: TransactionId, Err: MintError });
  const Stats = IDL.Record({
    transfers_count: IDL.Nat64,
    balance: IDL.Nat64,
    mints_count: IDL.Nat64,
    canisters_created_count: IDL.Nat64,
    supply: IDL.Nat,
    burns_count: IDL.Nat64,
    proxy_calls_count: IDL.Nat64,
    history_events: IDL.Nat64,
  });
  const TransferError = IDL.Variant({
    CallFailed: IDL.Null,
    InsufficientBalance: IDL.Null,
    Unknown: IDL.Null,
    AmountTooLarge: IDL.Null,
  });
  const TransferResponse = IDL.Variant({
    Ok: TransactionId,
    Err: TransferError,
  });
  const ResultCall = IDL.Variant({
    Ok: IDL.Record({ return: IDL.Vec(IDL.Nat8) }),
    Err: IDL.Text,
  });
  const CreateResult = IDL.Variant({
    Ok: IDL.Record({ canister_id: IDL.Principal }),
    Err: IDL.Text,
  });
  const ResultSend = IDL.Variant({ Ok: IDL.Null, Err: IDL.Text });
  return IDL.Service({
    balance: IDL.Func([IDL.Opt(IDL.Principal)], [IDL.Nat64], []),
    burn: IDL.Func(
      [IDL.Record({ canister_id: IDL.Principal, amount: IDL.Nat64 })],
      [BurnResult],
      []
    ),
    events: IDL.Func(
      [IDL.Record({ from: IDL.Opt(IDL.Nat64), limit: IDL.Nat16 })],
      [EventsConnection],
      ['query']
    ),
    get_transaction: IDL.Func([TransactionId], [IDL.Opt(Event)], []),
    halt: IDL.Func([], [], []),
    meta: IDL.Func([], [TokenMetaData], ['query']),
    meta_certified: IDL.Func([], [TokenMetaData], []),
    mint: IDL.Func([IDL.Opt(IDL.Principal)], [MintResult], []),
    name: IDL.Func([], [IDL.Opt(IDL.Text)], ['query']),
    stats: IDL.Func([], [Stats], ['query']),
    transfer: IDL.Func(
      [IDL.Record({ to: IDL.Principal, amount: IDL.Nat64 })],
      [TransferResponse],
      []
    ),
    wallet_balance: IDL.Func(
      [],
      [IDL.Record({ amount: IDL.Nat64 })],
      ['query']
    ),
    wallet_call: IDL.Func(
      [
        IDL.Record({
          args: IDL.Vec(IDL.Nat8),
          cycles: IDL.Nat64,
          method_name: IDL.Text,
          canister: IDL.Principal,
        }),
      ],
      [ResultCall],
      []
    ),
    wallet_create_canister: IDL.Func(
      [
        IDL.Record({
          controller: IDL.Opt(IDL.Principal),
          cycles: IDL.Nat64,
        }),
      ],
      [CreateResult],
      []
    ),
    wallet_create_wallet: IDL.Func(
      [
        IDL.Record({
          controller: IDL.Opt(IDL.Principal),
          cycles: IDL.Nat64,
        }),
      ],
      [CreateResult],
      []
    ),
    wallet_send: IDL.Func(
      [IDL.Record({ canister: IDL.Principal, amount: IDL.Nat64 })],
      [ResultSend],
      []
    ),
  });
};

// export const init = ({ IDL }) => { return []; };

module.exports = { idlFactory };
