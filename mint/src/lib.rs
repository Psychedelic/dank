use ic_cdk::export::candid::Principal;
use ic_cdk::*;
use ic_cdk_macros::*;

pub struct Ownership {
    owner: Principal,
    initial_balance: u64,
}

pub struct Data {
    current_owner: Option<Ownership>,
    controller: Principal,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            current_owner: None,
            controller: Principal::management_canister(),
        }
    }
}

#[init]
fn init() {
    let data = storage::get_mut::<Data>();
    data.controller = caller();
}

#[update]
async fn rent(owner: Principal) {
    let data = storage::get_mut::<Data>();
    assert_eq!(data.controller, caller());

    if let Some(ownership) = data.current_owner.take() {
        if let Err(e) = notify_dank(&data.controller, &ownership).await {
            data.current_owner = Some(ownership);
            trap(&format!("Failed {}", e));
        }
    }

    data.current_owner = Some(Ownership {
        owner,
        initial_balance: api::canister_balance(),
    });
}

#[update]
async fn notify() {
    let data = storage::get_mut::<Data>();

    if let Some(ownership) = data.current_owner.take() {
        if ownership.owner != caller() {
            trap("Only the current owner can call the notify method.");
        }

        if let Err(e) = notify_dank(&data.controller, &ownership).await {
            data.current_owner = Some(ownership);
            trap(&format!("Failed {}", e));
        }

        data.current_owner = None;
    }
}

#[inline]
async fn notify_dank(dank_id: &Principal, ownership: &Ownership) -> Result<u64, String> {
    let balance = api::canister_balance() - ownership.initial_balance;

    if balance == 0 {
        return Ok(0);
    }

    match api::call::call(dank_id.clone(), "notify_mint", (ownership.owner, balance)).await {
        Ok((transaction_id,)) => Ok(transaction_id),
        Err(e) => Err(format!(
            "Call failed with '{}', error-code = {:?}",
            e.1, e.0
        )),
    }
}
