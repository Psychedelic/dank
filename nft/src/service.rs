use crate::nft_ledger::NFSLedger;
use crate::types::*;
use crate::utils::*;

use ic_kit::macros::*;

#[update]
fn transfer(transfer_request: TransferRequest) -> TransferResponse {
    assert_ne!(
        transfer_request.from, transfer_request.to,
        "transfer request from and to cannot be the same"
    );
    assert_eq!(transfer_request.amount, 1, "only amount 1 is supported");
    expect_caller(&transfer_request.from.clone().into());

    ledger().transfer(
        &transfer_request.from,
        &transfer_request.to,
        &transfer_request.token,
    );

    Ok(Nat::from(1))
}

#[update]
fn mintNFT(mint_request: MintRequest) -> TokenIndex {
    expect_caller(&token_level_metadata().owner.expect("token owner not set"));

    ledger().mintNFT(&mint_request)
}

#[query]
fn bearer(token_identifier: TokenIdentifier) -> PrincipalReturn {
    ledger().bearer(&token_identifier)
}

#[query]
fn getAllMetadataForUser(user: User) -> Vec<TokenMetadata> {
    ledger().getAllMetadataForUser(&user)
}

#[query]
fn supply(token_identifier: TokenIdentifier) -> BalanceReturn {
    ledger().supply(&token_identifier)
}

#[query]
fn metadata(token_identifier: TokenIdentifier) -> MetadataReturn {
    ledger().metadata(&token_identifier)
}

#[init]
fn init(owner: Principal) {
    *token_level_metadata() = TokenLevelMetadata::new(Some(owner));
    TokenLevelMetadata::store_in_stable();
}

#[pre_upgrade]
fn preUpgrade() {
    NFSLedger::store_in_stable();
}

#[post_upgrade]
fn postUpgrade() {
    TokenLevelMetadata::restore_from_stable();
    NFSLedger::restore_from_stable();
}
