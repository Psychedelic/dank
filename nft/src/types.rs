use crate::utils::*;

use derive_new::*;
use ic_kit::candid::CandidType;
pub use ic_kit::candid::Nat;
pub use ic_kit::candid::Principal;
use ic_kit::Context;
use serde::Deserialize;

pub use std::convert::From;
pub use std::vec::Vec;

pub type Balance = Nat;
pub type Memo = String;
pub type SubAccount = Vec<u8>;
pub type TokenIdentifier = String;
pub type TokenIndex = u32;
pub type AccountIdentifier = String;

pub type PrincipalReturn = Result<Principal, CommonError>;
pub type BalanceReturn = Result<Balance, CommonError>;
pub type MetadataReturn = Result<Metadata, CommonError>;
pub type MetadataOptBlob = Option<Vec<u8>>;

#[derive(Clone, CandidType, Debug, Deserialize, Eq, Hash, PartialEq)]
pub enum User {
    address(AccountIdentifier),
    principal(Principal),
}

impl From<User> for Principal {
    fn from(user: User) -> Self {
        match user {
            User::principal(p) => p,
            _ => unimplemented!(),
        }
    }
}

impl From<Principal> for User {
    fn from(principal: Principal) -> Self {
        User::principal(principal)
    }
}

impl From<AccountIdentifier> for User {
    fn from(account_identifier: AccountIdentifier) -> Self {
        User::address(account_identifier)
    }
}

pub fn into_token_index(token_identifier: &TokenIdentifier) -> TokenIndex {
    token_identifier
        .parse::<u32>()
        .expect("unable to convert token identifier to token index")
}

pub fn into_token_identifier(token_index: &TokenIndex) -> TokenIdentifier {
    token_index.to_string()
}

#[derive(CandidType, Deserialize)]
pub struct TransferRequest {
    pub amount: Balance,
    pub from: User,
    pub memo: Memo,
    pub notify: bool,
    pub SubAccount: Option<SubAccount>,
    pub to: User,
    pub token: TokenIdentifier,
}

#[derive(Clone, CandidType, Deserialize)]
pub enum TransferError {
    CannotNotify(AccountIdentifier),
    InsufficientBalance,
    InvalidToken(TokenIdentifier),
    Other(String),
    Rejected,
    Unauthorized(AccountIdentifier),
}

pub type TransferResponse = Result<Balance, TransferError>;

#[derive(Clone, CandidType, Deserialize)]
pub struct MintRequest {
    pub metadata: MetadataOptBlob,
    pub to: User,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct FungibleMetadata {
    decimals: u8,
    metadata: MetadataOptBlob,
    name: String,
    symbol: String,
}

#[derive(Clone, CandidType, Deserialize, new)]
pub struct NonFungibleMetadata {
    metadata: MetadataOptBlob,
}

#[derive(Clone, CandidType, Deserialize)]
pub enum Metadata {
    fungible(FungibleMetadata),
    nonfugible(NonFungibleMetadata),
}

#[derive(Clone, CandidType, Deserialize)]
pub enum CommonError {
    InvalidToken(TokenIdentifier),
    Other(String),
}

#[derive(Clone, CandidType, Deserialize, new)]
pub struct TokenMetadata {
    pub principal: Principal,
    pub metadata: Option<Vec<u8>>,
    pub token_identifier: TokenIdentifier,
}

#[derive(new, CandidType, Default, Deserialize)]
pub struct TokenLevelMetadata {
    pub owner: Option<Principal>,
}

impl TokenLevelMetadata {
    pub fn store_in_stable() {
        ic_kit::get_context()
            .stable_store((std::mem::take(token_level_metadata()),))
            .expect("unable to store TokenLevelMetadata in stable storage");
    }

    pub fn restore_from_stable() {
        ic_kit::get_context()
            .stable_restore::<(TokenLevelMetadata,)>()
            .expect("unable to restore TokenLevelMetadata from stable storage");
    }
}
