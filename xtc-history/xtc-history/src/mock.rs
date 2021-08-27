use crate::backend::*;
use std::collections::BTreeMap;
use std::sync::Mutex;
use xtc_history_common::bucket::*;
use xtc_history_common::types::*;

const MOCK_BUCKET_CAPACITY: usize = 50;

pub struct MockBackend;
pub type MockCanisterId = u32;

type Map = BTreeMap<MockCanisterId, Option<BucketData<MockCanisterId>>>;

static mut STORAGE: Option<Mutex<Map>> = None;

#[inline]
fn storage() -> &'static mut Map {
    unsafe {
        if let Some(s) = &mut STORAGE {
            s.get_mut().unwrap()
        } else {
            let mut map = BTreeMap::new();
            // reserve index 0 for the main canister.
            map.insert(0, None);
            STORAGE = Some(Mutex::new(map));
            storage()
        }
    }
}

impl Backend<MockCanisterId> for MockBackend {
    fn create_canister() -> Res<MockCanisterId> {
        let storage = storage();
        let id = storage.len() as u32;
        storage.insert(id, None);
        Box::pin(async move { Ok(id) })
    }

    fn install_code(canister_id: &MockCanisterId) -> Res<()> {
        let storage = storage();
        storage.insert(*canister_id, Some(BucketData::default()));
        Box::pin(async move { Ok(()) })
    }

    fn write_metadata(
        canister_id: &MockCanisterId,
        data: SetBucketMetadataArgs<MockCanisterId>,
    ) -> Res<()> {
        let storage = storage();
        let bucket = storage
            .get_mut(canister_id)
            .expect("Canister not found.")
            .as_mut()
            .expect("Canister code not installed.");
        bucket.set_metadata(data);
        Box::pin(async move { Ok(()) })
    }

    fn append_transactions(canister_id: &MockCanisterId, data: &[Transaction]) -> Res<()> {
        let storage = storage();
        let bucket = storage
            .get_mut(canister_id)
            .expect("Canister not found.")
            .as_mut()
            .expect("Canister code not installed.");
        let res = if bucket.len() + data.len() > MOCK_BUCKET_CAPACITY {
            Err("Memory overflow.".to_string())
        } else {
            bucket.append(&mut data.into_iter().cloned().collect());
            Ok(())
        };
        Box::pin(async move { res })
    }

    fn lookup_transaction(
        canister_id: &MockCanisterId,
        id: TransactionId,
    ) -> Res<Option<Transaction>> {
        let storage = storage();
        let bucket = storage
            .get_mut(canister_id)
            .expect("Canister not found.")
            .as_mut()
            .expect("Canister code not installed.");
        let tx = bucket.get_transaction(id).cloned();
        Box::pin(async move { Ok(tx) })
    }

    fn id() -> MockCanisterId {
        0
    }
}
