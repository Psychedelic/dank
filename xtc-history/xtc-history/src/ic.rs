use crate::backend::*;
use ic_cdk::api::*;
use ic_cdk::export::candid::{encode_args, CandidType, Nat, Principal};
use serde::Deserialize;
use xtc_history_common::types::*;

pub struct IcBackend;

#[cfg(debug_cfg)]
const BUCKET_WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/debug/xtc_history_bucket-deb-opt.wasm");

#[cfg(not(debug_cfg))]
const BUCKET_WASM: &[u8] = include_bytes!(
    "../../../target/wasm32-unknown-unknown/release/xtc_history_bucket-rel-opt.wasm"
);

impl Backend<Principal> for IcBackend {
    fn create_canister() -> Res<Principal> {
        #[derive(Deserialize, CandidType)]
        struct CreateCanisterResult {
            canister_id: Principal,
        }

        #[derive(CandidType, Deserialize)]
        struct CanisterSettings {
            pub controller: Option<Principal>,
            pub compute_allocation: Option<Nat>,
            pub memory_allocation: Option<Nat>,
            pub freezing_threshold: Option<Nat>,
        }

        #[derive(CandidType)]
        struct In {
            settings: Option<CanisterSettings>,
        }

        Box::pin(async move {
            let mem: u64 = 4 * 1024 * 1024 * 1024;
            let in_arg = In {
                settings: Some(CanisterSettings {
                    controller: None,
                    compute_allocation: None,
                    memory_allocation: Some(Nat::from(mem)),
                    freezing_threshold: None,
                }),
            };

            let res: CreateCanisterResult = match call::call_with_payment(
                Principal::management_canister(),
                "create_canister",
                (in_arg,),
                50e12 as u64,
            )
            .await
            {
                Ok((data,)) => data,
                Err((code, msg)) => {
                    return Err(format!(
                        "An error happened during the call: {}: {}",
                        code as u8, msg
                    ))
                }
            };

            Ok(res.canister_id)
        })
    }

    fn install_code(canister_id: &Principal) -> Res<()> {
        #[derive(CandidType, Deserialize)]
        enum InstallMode {
            #[serde(rename = "install")]
            Install,
            #[serde(rename = "reinstall")]
            Reinstall,
            #[serde(rename = "upgrade")]
            Upgrade,
        }

        #[derive(CandidType, Deserialize)]
        struct CanisterInstall<'a> {
            mode: InstallMode,
            canister_id: Principal,
            #[serde(with = "serde_bytes")]
            wasm_module: &'a [u8],
            arg: Vec<u8>,
        }

        let canister_id = canister_id.clone();

        Box::pin(async move {
            let install_config = CanisterInstall {
                mode: InstallMode::Install,
                canister_id,
                wasm_module: BUCKET_WASM,
                arg: b" ".to_vec(),
            };

            match call::call(
                Principal::management_canister(),
                "install_code",
                (install_config,),
            )
            .await
            {
                Ok(x) => x,
                Err((code, msg)) => {
                    return Err(format!(
                        "An error happened during the call: {}: {}",
                        code as u8, msg
                    ));
                }
            };

            Ok(())
        })
    }

    fn write_metadata(
        canister_id: &Principal,
        metadata: SetBucketMetadataArgs<Principal>,
    ) -> Res<()> {
        let id = canister_id.clone();

        Box::pin(async move {
            match call::call(id, "set_metadata", (metadata,)).await {
                Ok(x) => x,
                Err((code, msg)) => {
                    return Err(format!(
                        "An error happened during the call: {}: {}",
                        code as u8, msg
                    ));
                }
            };

            Ok(())
        })
    }

    fn append_transactions(canister_id: &Principal, data: &[Transaction]) -> Res<()> {
        let id = canister_id.clone();
        let args_result = encode_args((data,));

        Box::pin(async move {
            let args_raw =
                args_result.map_err(|e| format!("Failed to encode arguments: {:?}", e))?;

            call::call_raw(id, "append", args_raw, 0)
                .await
                .map_err(|(code, msg)| {
                    format!(
                        "An error happened during the push call: {}: {}",
                        code as u8, msg
                    )
                })?;

            Ok(())
        })
    }

    fn lookup_transaction(canister_id: &Principal, id: TransactionId) -> Res<Option<Transaction>> {
        let canister_id = canister_id.clone();

        Box::pin(async move {
            let res: Option<Transaction> =
                match call::call(canister_id, "get_transaction", (id,)).await {
                    Ok((res,)) => res,
                    Err((code, msg)) => {
                        return Err(format!(
                            "An error happened during the push call: {}: {}",
                            code as u8, msg
                        ));
                    }
                };

            Ok(res)
        })
    }

    fn id() -> Principal {
        id()
    }
}
