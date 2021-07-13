use crate::history::{HistoryBuffer, Transaction, TransactionId, TransactionKind};
use crate::ledger::Ledger;
use ic_cdk::export::candid::Principal;
use ic_cdk::*;
use ic_cdk_macros::*;
use std::collections::{BTreeSet, VecDeque};

const NOTIFY_TIMEOUT: u64 = 10 * 60 * 1000; // 10 Min

pub struct MintingService {
    canister_ids: BTreeSet<Principal>,
    canisters: VecDeque<MintingCanister>,
}

struct MintingCanister {
    id: Principal,
    rented_at: Option<u64>,
}

impl Default for MintingService {
    fn default() -> Self {
        Self {
            canister_ids: BTreeSet::new(),
            canisters: VecDeque::new(),
        }
    }
}

impl MintingService {
    /// Create `additional` new canister
    pub async fn increase_pool_size(&mut self, additional: usize) {
        for _ in 0..additional {
            let id = loop {
                match deploy_canister().await {
                    Ok(address) => break address,
                    _ => {}
                }
            };

            self.canister_ids.insert(id.clone());
            self.canisters.push_front(MintingCanister {
                id,
                rented_at: None,
            });
        }
    }

    #[inline]
    pub async fn reserve(&mut self) -> Option<Principal> {
        match self.canisters.front() {
            Some(canister) if canister.rented_at.is_none() => {}
            Some(canister) if api::time() - canister.rented_at.unwrap() > NOTIFY_TIMEOUT => {}
            _ => return None,
        };

        let canister = self.canisters.pop_front().unwrap().id;

        match api::call::call(canister.clone(), "rent", (caller(),)).await {
            Ok(()) => self.canisters.push_back(MintingCanister {
                id: canister,
                rented_at: Some(api::time()),
            }),
            Err(_) => {
                self.canisters.push_front(MintingCanister {
                    id: canister,
                    rented_at: None,
                });
                return None;
            }
        }

        Some(canister)
    }
}

#[update]
async fn start_mint() -> Option<Principal> {
    let minting_service = storage::get_mut::<MintingService>();
    minting_service.reserve().await
}

#[update]
async fn notify_mint(account: Principal, cycles: u64) -> TransactionId {
    let canister = caller();
    let minting_service = storage::get_mut::<MintingService>();

    if !minting_service.canister_ids.contains(&canister) {
        trap("Not allowed.");
    }

    let mut entries = Vec::with_capacity(10);

    loop {
        let mut entry = minting_service.canisters.pop_back().unwrap();

        if entry.id == canister {
            entry.rented_at = None;
            minting_service.canisters.push_front(entry);
            break;
        }

        entries.push(entry);
    }

    for entry in entries.into_iter().rev() {
        minting_service.canisters.push_back(entry);
    }

    let ledger = storage::get_mut::<Ledger>();
    let balance = ledger.0.entry(account).or_insert(0);
    *balance += cycles;

    let transaction = Transaction {
        timestamp: api::time(),
        cycles,
        fee: 0,
        kind: TransactionKind::Deposit { to: account },
    };

    storage::get_mut::<HistoryBuffer>().push(transaction)
}

async fn deploy_canister() -> Result<Principal, String> {
    unimplemented!()
}
