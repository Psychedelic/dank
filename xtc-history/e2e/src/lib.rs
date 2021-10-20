use ic_cdk::export::candid::Principal;
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use xtc_history::ic::IcBackend;
use xtc_history::History;
use xtc_history_common::types::*;

struct CanisterData {
    history: History,
    cursor: u32,
}

impl Default for CanisterData {
    fn default() -> Self {
        CanisterData {
            history: History::<Principal, IcBackend>::new(100, 25),
            cursor: 0,
        }
    }
}

#[update]
async fn insert(count: u32, progress: bool) {
    let data = storage::get_mut::<CanisterData>();

    if progress {
        data.history.progress().await;
    }

    for _ in 0..count {
        let index = data.cursor;
        data.cursor += 1;

        let event = Transaction {
            timestamp: api::time() / 1_000_000,
            cycles: index as u64,
            fee: 0,
            kind: TransactionKind::Mint {
                to: Principal::anonymous(),
            },
            status: TransactionStatus::SUCCEEDED,
        };

        data.history.push(event);
    }
}

#[update]
async fn stabilize() {
    let data = storage::get_mut::<CanisterData>();
    while data.history.progress().await {}
}

#[update]
async fn get_transaction(id: TransactionId) -> Option<Transaction> {
    let data = storage::get_mut::<CanisterData>();
    data.history.get_transaction(id).await
}

#[query]
fn events(args: EventsArgs) -> EventsConnection<'static> {
    let offset = args.offset;
    let limit = args.limit.min(512);

    storage::get::<CanisterData>().history.events(offset, limit)
}
