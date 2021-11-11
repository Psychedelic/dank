use crate::utils::convert_int_to_u64;
use cap_sdk::IndefiniteEvent;
use cap_sdk::{DetailValue, Event};
use derive_builder::*;
use derive_new::*;
use ic_kit::{
    candid::{CandidType, Int, Nat},
    Principal,
};
use std::collections::{HashMap, VecDeque};
use std::convert::{From, Into, TryFrom, TryInto};
use xtc_history_common::types::*;

type Time = Int;

#[derive(CandidType, Clone)]
pub enum Operation {
    approve,
    mint,
    transfer,
    transferFrom,
    burn,
    canisterCalled,
    canisterCreated,
}

#[derive(CandidType)]
pub struct Metadata<'a> {
    pub decimals: u8,
    pub fee: Nat,
    pub logo: &'a str,
    pub name: &'a str,
    pub owner: Principal,
    pub symbol: &'a str,
    pub totalSupply: Nat,
}

#[derive(CandidType, Debug, Eq, PartialEq)]
pub enum TxError {
    InsufficientAllowance,
    InsufficientBalance,
    Other,
}

pub type TxReceipt = Result<Nat, TxError>;

#[derive(Default)]
pub struct TxLog {
    pub tx_records: VecDeque<TxRecordExt>,
}

#[derive(CandidType, Clone, new)]
pub struct TxRecord {
    pub caller: Option<Principal>,
    pub from: Principal,
    pub to: Principal,
    pub amount: Nat,
    pub fee: Nat,
    pub op: Operation,
    pub timestamp: Time,
    pub index: Nat,
    pub status: TransactionStatus,
}

#[derive(CandidType, Clone, new)]
pub struct TxRecordExt {
    pub caller: Option<Principal>,
    pub from: Principal,
    pub to: Principal,
    pub amount: Nat,
    pub fee: Nat,
    pub op: Operation,
    pub timestamp: Time,
    pub index: Nat,
    pub status: TransactionStatus,
    pub method_name: String,
}

impl TxRecord {
    pub fn setIndex(mut self, index: Nat) -> Self {
        self.index = index;
        self
    }
}

// TODO CAP: can be removed from below, if CAP SDK supports it
// if CAP SDK will not support Dank legacy interface, then convert Transaction to TxRecord before
// calling CAP

fn create_tx_record(
    operation: Operation,
    event: &Event,
    details: HashMap<&str, DetailValue>,
) -> Result<TxRecord, ()> {
    Ok(TxRecord::new(
        None,
        event.caller,
        details.get("to").unwrap().clone().try_into()?,
        details.get("amount").unwrap().clone().try_into()?,
        details.get("fee").unwrap().clone().try_into()?,
        operation,
        Int::from(event.time),
        Nat::from(0),
        convert_string_to_transaction_status(
            TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
        ),
    ))
}

impl TryFrom<Event> for TxRecord {
    type Error = ();
    fn try_from(event: Event) -> Result<TxRecord, ()> {
        let mut details: HashMap<&str, DetailValue> = HashMap::new();
        for detail in &event.details {
            details.insert(&detail.0, detail.1.clone());
        }
        match event.operation.as_str() {
            "approve" => Ok(create_tx_record(Operation::approve, &event, details)?),
            "transfer" => Ok(create_tx_record(Operation::transfer, &event, details)?),
            "transferFrom" => {
                let has_from = details.get("from").is_some();

                Ok(Self::new(
                    details
                        .get("from")
                        .map(|from| from.clone().try_into().unwrap()),
                    event.caller,
                    details.get("to").unwrap().clone().try_into()?,
                    details.get("amount").unwrap().clone().try_into()?,
                    details.get("fee").unwrap().clone().try_into()?,
                    Operation::transferFrom,
                    Int::from(event.time),
                    Nat::from(0),
                    convert_string_to_transaction_status(
                        TryInto::<String>::try_into(details.get("status").unwrap().clone())
                            .unwrap(),
                    ),
                ))
            }
            "mint" => Ok(create_tx_record(Operation::mint, &event, details)?),
            "burn" => Ok(create_tx_record(Operation::burn, &event, details)?),
            "canisterCalled" => Ok(create_tx_record(
                Operation::canisterCalled,
                &event,
                details,
            )?),
            "canisterCreated" => Ok(create_tx_record(
                Operation::canisterCreated,
                &event,
                details,
            )?),
            &_ => Err(()),
        }
    }
}

// todo convert this to TryInto when Transaction has been moved to xtc crate
// impl From<String> for TransactionStatus {
//     fn from(event_status: String) -> Self {
//         match event_status.as_str() {
//             "inprogress" => TransactionStatus::INPROGRESS,
//             "succeeded" => TransactionStatus::SUCCEEDED,
//             "failed" => TransactionStatus::FAILED,
//             &_ => panic!("unknow transaction status"),
//         }
//     }
// }

// impl From<TransactionStatus> for String {
//     fn from(transaction_status: TransactionStatus) -> Self {
//         match transaction_status {
//             TransactionStatus::INPROGRESS => "inprogress".to_string(),
//             TransactionStatus::SUCCEEDED => "succeeded".to_string(),
//             TransactionStatus::FAILED => "failed".to_string(),
//         }
//     }
// }

fn convert_string_to_transaction_status(event_status: String) -> TransactionStatus {
    match event_status.as_str() {
        "inprogress" => TransactionStatus::INPROGRESS,
        "succeeded" => TransactionStatus::SUCCEEDED,
        "failed" => TransactionStatus::FAILED,
        &_ => panic!("unknow transaction status"),
    }
}

fn convert_transaction_status_to_string(transaction_status: &TransactionStatus) -> String {
    match transaction_status {
        TransactionStatus::INPROGRESS => "inprogress".to_string(),
        TransactionStatus::SUCCEEDED => "succeeded".to_string(),
        TransactionStatus::FAILED => "failed".to_string(),
    }
}

fn create_event(
    record: &TxRecordExt,
    operation_name: &str,
    mut details: Vec<(String, DetailValue)>,
) -> Event {
    details.append(&mut vec![
        ("to".to_string(), record.to.into()),
        ("amount".to_string(), record.amount.clone().into()),
        ("fee".to_string(), record.fee.clone().into()),
        (
            "status".to_string(),
            convert_transaction_status_to_string(&record.status.clone()).into(),
        ),
    ]);
    Event {
        time: convert_int_to_u64(record.timestamp.clone()).expect("Int -> Nat64 conversion failed"),
        caller: record.from.clone(),
        operation: operation_name.to_string(),
        details,
    }
}

impl TryFrom<TxRecordExt> for Event {
    type Error = ();
    fn try_from(record: TxRecordExt) -> Result<Event, ()> {
        Ok(match record.op {
            Operation::approve => create_event(&record, "approve", vec![]),
            Operation::transfer => create_event(&record, "transfer", vec![]),
            Operation::transferFrom => {
                let mut event = Event {
                    time: convert_int_to_u64(record.timestamp.clone())
                        .expect("Int -> Nat64 conversion failed"),
                    caller: record.from.clone(),
                    operation: "transferFrom".to_string(),
                    details: vec![
                        ("to".to_string(), record.to.into()),
                        ("amount".to_string(), record.amount.clone().into()),
                        ("fee".to_string(), record.fee.clone().into()),
                        (
                            "status".to_string(),
                            convert_transaction_status_to_string(&record.status.clone()).into(),
                        ),
                    ],
                };
                if let Some(caller) = record.caller {
                    event.details.push(("from".to_string(), caller.into()));
                }
                event
            }
            Operation::mint => create_event(&record, "mint", vec![]),
            Operation::burn => create_event(&record, "burn", vec![]),
            Operation::canisterCalled => create_event(
                &record,
                "canisterCalled",
                vec![(
                    "method_name".to_string(),
                    record.method_name.to_string().into(),
                )],
            ),
            Operation::canisterCreated => create_event(&record, "canisterCreated", vec![]),
        })
    }
}

pub fn event_into_indefinite_event(event: &Event) -> IndefiniteEvent {
    IndefiniteEvent {
        caller: event.caller.clone(),
        operation: event.operation.clone(),
        details: event.details.clone(),
    }
}

// todo convert this to TryInto when Transaction has been moved to xtc crate
fn convert_event_to_transaction(event: &Event) -> Result<Transaction, ()> {
    let mut details: HashMap<&str, DetailValue> = HashMap::new();
    for detail in &event.details {
        details.insert(&detail.0, detail.1.clone());
    }
    match event.operation.as_str() {
        "transfer" => Ok(Transaction {
            timestamp: event.time,
            cycles: details.get("amount").unwrap().clone().try_into()?,
            fee: details.get("fee").unwrap().clone().try_into()?,
            kind: TransactionKind::Transfer {
                from: event.caller,
                to: details.get("to").unwrap().clone().try_into()?,
            },
            status: convert_string_to_transaction_status(
                TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
            ),
        }),
        "mint" => Ok(Transaction {
            timestamp: event.time,
            cycles: details.get("amount").unwrap().clone().try_into()?,
            fee: details.get("fee").unwrap().clone().try_into()?,
            kind: TransactionKind::Mint {
                to: details.get("to").unwrap().clone().try_into()?,
            },
            status: convert_string_to_transaction_status(
                TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
            ),
        }),
        "brun" => Ok(Transaction {
            timestamp: event.time,
            cycles: details.get("amount").unwrap().clone().try_into()?,
            fee: details.get("fee").unwrap().clone().try_into()?,
            kind: TransactionKind::Burn {
                from: event.caller,
                to: details.get("to").unwrap().clone().try_into()?,
            },
            status: convert_string_to_transaction_status(
                TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
            ),
        }),
        "approve" => Ok(Transaction {
            timestamp: event.time,
            cycles: details.get("amount").unwrap().clone().try_into()?,
            fee: details.get("fee").unwrap().clone().try_into()?,
            kind: TransactionKind::Approve {
                from: event.caller,
                to: details.get("to").unwrap().clone().try_into()?,
            },
            status: convert_string_to_transaction_status(
                TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
            ),
        }),
        "canisterCalled" => Ok(Transaction {
            timestamp: event.time,
            cycles: details.get("amount").unwrap().clone().try_into()?,
            fee: details.get("fee").unwrap().clone().try_into()?,
            kind: TransactionKind::CanisterCalled {
                from: event.caller,
                canister: details.get("to").unwrap().clone().try_into()?,
                method_name: details.get("method_name").unwrap().clone().try_into()?,
            },
            status: convert_string_to_transaction_status(
                TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
            ),
        }),
        "canisterCreated" => Ok(Transaction {
            timestamp: event.time,
            cycles: details.get("amount").unwrap().clone().try_into()?,
            fee: details.get("fee").unwrap().clone().try_into()?,
            kind: TransactionKind::CanisterCreated {
                from: event.caller,
                canister: details.get("to").unwrap().clone().try_into()?,
            },
            status: convert_string_to_transaction_status(
                TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
            ),
        }),
        "transferFrom" => {
            let from_field = details.get("from");

            Ok(Transaction {
                timestamp: event.time,
                cycles: details.get("amount").unwrap().clone().try_into()?,
                fee: details.get("fee").unwrap().clone().try_into()?,
                kind: TransactionKind::TransferFrom {
                    from: event.caller,
                    caller: if from_field.is_some() {
                        from_field.unwrap().clone().try_into()?
                    } else {
                        event.caller
                    },
                    to: details.get("to").unwrap().clone().try_into()?,
                },
                status: convert_string_to_transaction_status(
                    TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
                ),
            })
        }
        "approve" => Ok(Transaction {
            timestamp: event.time,
            cycles: details.get("amount").unwrap().clone().try_into()?,
            fee: details.get("fee").unwrap().clone().try_into()?,
            kind: TransactionKind::Approve {
                from: event.caller,
                to: details.get("to").unwrap().clone().try_into()?,
            },
            status: convert_string_to_transaction_status(
                TryInto::<String>::try_into(details.get("status").unwrap().clone()).unwrap(),
            ),
        }),
        &_ => Err(()),
    }
}

fn convert_transaction_to_event(transaction: &Transaction) -> Result<Event, ()> {
    match &transaction.kind {
        TransactionKind::Transfer { from, to } => Ok(Event {
            time: transaction.timestamp,
            caller: from.clone(),
            operation: "transfer".to_string(),
            details: vec![
                ("to".to_string(), DetailValue::Principal(to.clone())),
                ("amount".to_string(), transaction.cycles.into()),
                ("fee".to_string(), transaction.fee.into()),
                (
                    "status".to_string(),
                    convert_transaction_status_to_string(&transaction.status).into(),
                ),
            ],
        }),
        TransactionKind::Mint { to } => Ok(Event {
            time: transaction.timestamp,
            caller: to.clone(),
            operation: "mint".to_string(),
            details: vec![
                ("to".to_string(), DetailValue::Principal(to.clone())),
                ("amount".to_string(), transaction.cycles.into()),
                ("fee".to_string(), transaction.fee.into()),
                (
                    "status".to_string(),
                    convert_transaction_status_to_string(&transaction.status).into(),
                ),
            ],
        }),
        TransactionKind::Burn { from, to } => Ok(Event {
            time: transaction.timestamp,
            caller: from.clone(),
            operation: "burn".to_string(),
            details: vec![
                ("to".to_string(), DetailValue::Principal(to.clone())),
                ("amount".to_string(), transaction.cycles.into()),
                ("fee".to_string(), transaction.fee.into()),
                (
                    "status".to_string(),
                    convert_transaction_status_to_string(&transaction.status).into(),
                ),
            ],
        }),
        TransactionKind::CanisterCalled {
            from,
            canister,
            method_name,
        } => Ok(Event {
            time: transaction.timestamp,
            caller: from.clone(),
            operation: "canisterCalled".to_string(),
            details: vec![
                ("to".to_string(), DetailValue::Principal(canister.clone())),
                ("amount".to_string(), transaction.cycles.into()),
                ("fee".to_string(), transaction.fee.into()),
                (
                    "status".to_string(),
                    convert_transaction_status_to_string(&transaction.status).into(),
                ),
                ("method_name".to_string(), method_name.clone().into()),
            ],
        }),
        TransactionKind::CanisterCreated { from, canister } => Ok(Event {
            time: transaction.timestamp,
            caller: from.clone(),
            operation: "canisterCreated".to_string(),
            details: vec![
                ("to".to_string(), DetailValue::Principal(canister.clone())),
                ("amount".to_string(), transaction.cycles.into()),
                ("fee".to_string(), transaction.fee.into()),
                (
                    "status".to_string(),
                    convert_transaction_status_to_string(&transaction.status).into(),
                ),
            ],
        }),
        TransactionKind::Approve { from, to } => Ok(Event {
            time: transaction.timestamp,
            caller: from.clone(),
            operation: "approve".to_string(),
            details: vec![
                ("to".to_string(), DetailValue::Principal(to.clone())),
                ("amount".to_string(), transaction.cycles.into()),
                ("fee".to_string(), transaction.fee.into()),
                (
                    "status".to_string(),
                    convert_transaction_status_to_string(&transaction.status).into(),
                ),
            ],
        }),
        TransactionKind::TransferFrom { caller, from, to } => {
            let mut details = vec![
                ("to".to_string(), DetailValue::Principal(to.clone())),
                ("amount".to_string(), transaction.cycles.into()),
                ("fee".to_string(), transaction.fee.into()),
                (
                    "status".to_string(),
                    convert_transaction_status_to_string(&transaction.status).into(),
                ),
            ];
            if from != caller {
                details.push(("from".to_string(), caller.clone().into()));
            }
            Ok(Event {
                time: transaction.timestamp,
                caller: from.clone(),
                operation: "transfer".to_string(),
                details,
            })
        }
    }
}
