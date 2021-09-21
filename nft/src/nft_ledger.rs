use crate::types::*;
use crate::utils::*;

use derive_new::*;
use ic_kit::candid::CandidType;
use ic_kit::ic::trap;
use ic_kit::Context;
use serde::Deserialize;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(CandidType, Default, Deserialize)]
pub struct NFSLedger {
    tokens: HashMap<TokenIndex, TokenMetadata>,
    user_tokens: HashMap<User, Vec<TokenIndex>>,
}

impl NFSLedger {
    pub fn transfer(&mut self, from: &User, to: &User, token_identifier: &TokenIdentifier) {
        // changeing token owner in the tokens map
        let token_index = into_token_index(token_identifier);
        ledger()
            .tokens
            .get_mut(&token_index)
            .expect("unable to find token identifier in tokens")
            .principal = to.clone().into();

        // remove the token from the previous owner's tokenlist
        let mut from_token_indexes = ledger()
            .user_tokens
            .get_mut(&from)
            .expect("unable to find previous owner");
        from_token_indexes.remove(
            from_token_indexes
                .iter()
                .position(|token_index_in_vec| &token_index == token_index_in_vec)
                .expect("unable to find token index in users_token"),
        );
        if (from_token_indexes.len() == 0) {
            ledger().user_tokens.remove(&from);
        }

        // add the token to the new owner's tokenlist
        ledger()
            .user_tokens
            .entry(to.clone())
            .or_default()
            .push(token_index);
    }

    pub fn mintNFT(&mut self, mint_request: &MintRequest) -> TokenIndex {
        let token_index = ledger().tokens.len() as TokenIndex;
        ledger().tokens.insert(
            token_index,
            TokenMetadata::new(
                caller(),
                mint_request.metadata.clone(),
                into_token_identifier(&token_index),
            ),
        );
        ledger()
            .user_tokens
            .entry(caller().into())
            .or_default()
            .push(token_index);
        token_index
    }

    pub fn bearer(&self, token_identifier: &TokenIdentifier) -> PrincipalReturn {
        PrincipalReturn::Ok(
            ledger()
                .tokens
                .get(&into_token_index(&token_identifier))
                .expect("unable to locate token id")
                .principal,
        )
    }

    pub fn getAllMetadataForUser(&self, user: &User) -> Vec<TokenMetadata> {
        ledger()
            .user_tokens
            .get(user)
            .expect("unable to find user")
            .iter()
            .map(|token_index| {
                ledger()
                    .tokens
                    .get(token_index)
                    .expect("unable to find token index")
                    .clone()
            })
            .collect()
    }

    pub fn supply(&self, token_identifier: &TokenIdentifier) -> BalanceReturn {
        BalanceReturn::Ok(ledger().tokens.len().into())
    }

    pub fn metadata(&self, token_identifier: &TokenIdentifier) -> MetadataReturn {
        MetadataReturn::Ok(Metadata::nonfugible(NonFungibleMetadata::new(
            ledger()
                .tokens
                .get(&into_token_index(&token_identifier))
                .expect("unable to find token index")
                .metadata
                .clone(),
        )))
    }

    pub fn store_in_stable() {
        ic_kit::get_context()
            .stable_store((std::mem::take(ledger()),))
            .expect("unable to store NFTLedger in stable storage");
    }

    pub fn restore_from_stable() {
        ic_kit::get_context()
            .stable_restore::<(NFSLedger,)>()
            .expect("unable to restore NFTLedger from stable storage");
    }
}
