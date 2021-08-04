mod cycles_wallet;
mod history;
mod ledger;
mod management;
mod meta;
mod stats;
mod upgrade;

/// Perform only one pending async task, returns whether an async call was performed
/// as the result of calling this method or not.
/// This method should only be called from updates.
///
/// Currently only the history has async tasks, but in future there might
/// be more things following this design pattern for handling tasks.
#[inline]
pub async fn progress() -> bool {
    use ic_cdk::storage;

    let history = storage::get_mut::<history::HistoryBuffer>();
    history.progress().await
}
