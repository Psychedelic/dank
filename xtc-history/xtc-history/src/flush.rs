use crate::backend::Backend;
use crate::data::HistoryData;
use ic_cdk::*;
use std::marker::PhantomData;

pub struct HistoryFlusher<Address, Storage: Backend<Address>> {
    state: State<Address>,
    chunk_size: usize,
    in_progress: bool,
    backend: PhantomData<(Address, Storage)>,
}

pub enum ProgressResult {
    /// Progress has been made.
    Ok,
    /// The progress call returned because another parallel progress is currently executing.
    Blocked,
    /// The flush has completed, there are no more data to write to the buckets.
    Done,
}

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum State<Address> {
    /// Make the call to the management canister to create a new canister.
    ///
    /// Next : InstallCode { canister_id }
    CreateCanister,
    /// Install the bucket canister's WASM to the given canister id.
    ///
    /// Next: WriteMetadata { canister_id }
    InstallCode { canister_id: Address },
    /// Writes the meta-data to the bucket.
    ///
    /// Next: PushChunk
    WriteMetadata { canister_id: Address },
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

impl<Address: Clone + std::cmp::PartialEq, Storage: Backend<Address>>
    HistoryFlusher<Address, Storage>
{
    pub fn new(bucket_exists: bool, chunk_size: usize) -> Self {
        HistoryFlusher {
            state: match bucket_exists {
                true => State::PushChunk,
                false => State::CreateCanister,
            },
            chunk_size,
            in_progress: false,
            backend: PhantomData::default(),
        }
    }

    pub async fn progress(&mut self, data: &mut HistoryData<Address>) -> ProgressResult {
        if State::Done == self.state {
            return ProgressResult::Done;
        }

        // Guard against parallel execution.
        if self.in_progress {
            return ProgressResult::Blocked;
        }

        self.in_progress = true;

        match &self.state {
            State::CreateCanister => {
                match Storage::create_canister().await {
                    Ok(canister_id) => {
                        self.state = State::InstallCode { canister_id };
                    }
                    Err(e) => {
                        api::print(e);
                    }
                };
            }
            State::InstallCode { canister_id } => {
                match Storage::install_code(canister_id).await {
                    Ok(()) => {
                        self.state = State::WriteMetadata {
                            canister_id: canister_id.clone(),
                        };
                    }
                    Err(e) => {
                        api::print(e);
                    }
                };
            }
            State::WriteMetadata { canister_id } => {
                let metadata = data.get_metadata();

                match Storage::write_metadata(canister_id, metadata).await {
                    Ok(()) => {
                        data.insert_bucket(canister_id.clone());
                        self.state = State::PushChunk;
                    }
                    Err(e) => {
                        api::print(e);
                    }
                };
            }
            State::PushChunk => {
                // Data we need to write.
                let chunk = &data.get_events()[0..self.chunk_size];
                // The bucket canister we need to write the data to.
                let canister_id = data.get_bucket();

                self.state = match Storage::append_transactions(canister_id, chunk).await {
                    Ok(()) => {
                        data.remove_first(self.chunk_size);

                        if data.len() < self.chunk_size {
                            State::Done
                        } else {
                            State::PushChunk
                        }
                    }
                    Err(_) => {
                        // TODO(qti3e) Only move to create canister state if the error returned
                        // is because of memory allocation errors.
                        State::CreateCanister
                    }
                };
            }
            State::Done => unreachable!(),
        }

        self.in_progress = false;
        ProgressResult::Ok
    }
}
