use ic_cdk::export::candid::{CandidType, Nat, Principal};
use ic_cdk::*;
use serde::Deserialize;
use xtc_history_types::*;

const BUCKET_WASM: &[u8] =
    include_bytes!("../../../target/wasm32-unknown-unknown/release/xtc_history_bucket-opt.wasm");

pub struct HistoryFlusher {
    state: State,
    chunk_size: usize,
    in_progress: bool,
    offset: TransactionId,
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
        next_event_id: TransactionId,
        current_buffer_len: usize,
        bucket_exists: bool,
        chunk_size: usize,
    ) -> Self {
        HistoryFlusher {
            state: match bucket_exists {
                true => State::PushChunk,
                false => State::CreateCanister,
            },
            chunk_size,
            in_progress: false,
            offset: next_event_id - current_buffer_len as u64,
        }
    }

    pub async fn progress(
        &mut self,
        buckets: &mut Vec<(TransactionId, Principal)>,
        data: &mut Vec<Transaction>,
    ) -> ProgressResult {
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
                        from: self.offset,
                        next: match buckets.is_empty() {
                            true => None,
                            false => Some(buckets[buckets.len() - 1].1.clone()),
                        },
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

                    buckets.push((self.offset, canister_id));
                    self.state = State::PushChunk;
                }
                State::PushChunk => {
                    if data.len() < self.chunk_size {
                        self.state = State::Done;
                        break;
                    }

                    // Data we need to write.
                    let chunk = &data[0..self.chunk_size];
                    // The bucket canister we need to write the data to.
                    let canister_id = buckets[buckets.len() - 1].1.clone();

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

                    self.offset += self.chunk_size as u64;
                    data.drain(0..self.chunk_size);

                    self.state = if data.len() < self.chunk_size {
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
