use ic_cdk::*;
use ic_cdk_macros::*;

mod history;
mod ledger;
mod minting;
mod proxy;
mod upgrade;

#[init]
async fn init() {
    let minting_service = storage::get_mut::<minting::MintingService>();
    minting_service.increase_pool_size(10).await;
}
