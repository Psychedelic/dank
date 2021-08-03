use ic_cdk::export::Principal;
use xtc_history_types::*;

pub struct HistoryFlusher {
    state: State,
    head: Option<Principal>,
    data: Vec<Transaction>,
    chunk_size: usize,
    in_progress: bool,
    cursor: usize,
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
    pub fn new(data: Vec<Transaction>, head: Option<Principal>, chunk_size: usize) -> Self {
        HistoryFlusher {
            state: match head {
                Some(_) => State::PushChunk,
                None => State::CreateCanister,
            },
            head,
            data,
            chunk_size,
            in_progress: false,
            cursor: 0,
        }
    }

    pub async fn progress(&mut self) -> ProgressResult {
        if State::Done == self.state {
            return ProgressResult::Done;
        }

        // Guard against parallel execution.
        if self.in_progress {
            return ProgressResult::Blocked;
        }

        self.in_progress = true;

        match self.state {
            State::CreateCanister => {
                todo!()
            }
            State::InstallCode { canister_id } => {
                todo!()
            }
            State::WriteMetadata { canister_id } => {
                todo!()
            }
            State::PushChunk => {
                todo!()
            }
            State::Done => unreachable!(),
        };

        self.in_progress = false;
        ProgressResult::Ok
    }
}
