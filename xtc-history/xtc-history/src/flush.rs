use crate::BucketsList;
use ic_cdk::export::candid::{CandidType, Nat, Principal};
use ic_cdk::*;
use serde::Deserialize;
use xtc_history_types::*;

const BUCKET_WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/xtc_history_bucket-opt.wasm");

pub struct HistoryFlusher {
    state: State,
    pub(crate) data: Vec<Transaction>,
    chunk_size: usize,
    in_progress: bool,
    cursor: usize,
    global_cursor: TransactionId,
}

pub enum ProgressResult {
    /// Progress has been made.
    Ok,
    /// The progress call returned because another parallel progress is currently executing.
    Blocked,
    /// The flush has completed, there are no more data to write to the buckets.
    Done,
}

#[derive(PartialOrd, PartialEq)]
enum State {
    /// Make the call to the management canister to create a new canister.
    ///
    /// Next : InstallCode { canister_id }
    CreateCanister,
    /// Install the bucket canister's WASM to the given canister id.
    ///
    /// Next: WriteMetadata { canister_id }
    InstallCode { canister_id: Principal },
    /// Writes the meta-data to the bucket.
    ///
    /// Next: PushChunk
    WriteMetadata { canister_id: Principal },
    /// Tries to write a chunk of events to the head archive canister.
    ///
    /// Next: PushChunk
    /// default
    ///
    /// Next: Done
    /// If there is no more chunk to write.
    ///
    /// Next: CreateCanister
    /// If the call failed.
    PushChunk,
    /// Final state of the system.
    Done,
}

impl HistoryFlusher {
    pub fn new(
        data: Vec<Transaction>,
        head: Option<Principal>,
        global_cursor: TransactionId,
        chunk_size: usize,
    ) -> Self {
        HistoryFlusher {
            state: match head {
                Some(_) => State::PushChunk,
                None => State::CreateCanister,
            },
            data,
            chunk_size,
            global_cursor,
            in_progress: false,
            cursor: 0,
        }
    }

    pub async fn progress(&mut self, buckets: &mut BucketsList) -> ProgressResult {
        if State::Done == self.state {
            return ProgressResult::Done;
        }

        // Guard against parallel execution.
        if self.in_progress {
            return ProgressResult::Blocked;
        }

        self.in_progress = true;

        loop {
            match self.state {
                State::CreateCanister => {
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

                    let in_arg = In {
                        settings: Some(CanisterSettings {
                            controller: None,
                            compute_allocation: None,
                            memory_allocation: Some(Nat::from(4 * 1024 * 1024 * 1024)),
                            freezing_threshold: None,
                        }),
                    };

                    let res: CreateCanisterResult = match api::call::call_with_payment(
                        Principal::management_canister(),
                        "create_canister",
                        (in_arg,),
                        50e12 as u64,
                    )
                    .await
                    {
                        Ok((data,)) => data,
                        Err((code, msg)) => {
                            api::print(format!(
                                "An error happened during the call: {}: {}",
                                code as u8, msg
                            ));
                            // Ignore the error, don't change state, try again later.
                            break;
                        }
                    };

                    self.state = State::InstallCode {
                        canister_id: res.canister_id,
                    };
                }
                State::InstallCode { canister_id } => {
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

                    let install_config = CanisterInstall {
                        mode: InstallMode::Install,
                        canister_id: canister_id.clone(),
                        wasm_module: BUCKET_WASM,
                        arg: b" ".to_vec(),
                    };

                    match api::call::call(
                        Principal::management_canister(),
                        "install_code",
                        (install_config,),
                    )
                    .await
                    {
                        Ok(x) => x,
                        Err((code, msg)) => {
                            api::print(format!(
                                "An error happened during the call: {}: {}",
                                code as u8, msg
                            ));
                            break;
                        }
                    };

                    self.state = State::WriteMetadata { canister_id };
                }
                State::WriteMetadata { canister_id } => {
                    let metadata = SetBucketMetadataArgs {
                        from: self.global_cursor,
                        next: buckets.get_head().cloned(),
                    };

                    match api::call::call(canister_id.clone(), "set_metadata", (metadata,)).await {
                        Ok(x) => x,
                        Err((code, msg)) => {
                            api::print(format!(
                                "An error happened during the call: {}: {}",
                                code as u8, msg
                            ));
                            break;
                        }
                    };

                    buckets.insert(canister_id, self.global_cursor);
                    self.state = State::PushChunk;
                }
                State::PushChunk => {
                    let chunk = &self.data[self.cursor..self.cursor + self.chunk_size];
                    let canister_id = buckets.get_head().cloned().unwrap();

                    match api::call::call(canister_id, "push", (chunk,)).await {
                        Ok(x) => x,
                        Err((code, msg)) => {
                            api::print(format!(
                                "An error happened during the push call: {}: {}",
                                code as u8, msg
                            ));
                            // Create a new archive.
                            self.state = State::CreateCanister;
                            break;
                        }
                    };

                    self.cursor += self.chunk_size;
                    self.global_cursor += self.chunk_size as TransactionId;

                    self.state = if self.cursor >= self.data.len() {
                        State::Done
                    } else {
                        State::PushChunk
                    };
                }
                State::Done => unreachable!(),
            };

            break;
        }

        self.in_progress = false;
        ProgressResult::Ok
    }
}
