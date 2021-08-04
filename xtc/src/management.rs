use ic_cdk::export::candid::Principal;
use ic_cdk::*;
use ic_cdk_macros::*;

// --- init

pub struct Controller(Option<Principal>);

impl Default for Controller {
    fn default() -> Self {
        Controller(None)
    }
}

impl Controller {
    pub fn load(controller: Principal) {
        let data = storage::get_mut::<Controller>();
        data.0 = Some(controller);
    }

    pub fn load_if_not_present(controller: Principal) {
        let data = storage::get_mut::<Controller>();
        if data.0.is_none() {
            data.0 = Some(controller);
        }
    }

    #[inline]
    pub fn get_principal() -> Principal {
        storage::get::<Controller>().0.unwrap().clone()
    }
}

#[init]
fn init() {
    Controller::load_if_not_present(caller());
}

// --- halt

pub struct IsShutDown(bool);

impl Default for IsShutDown {
    fn default() -> Self {
        IsShutDown(false)
    }
}

impl IsShutDown {
    #[inline]
    pub fn get() -> bool {
        storage::get::<IsShutDown>().0
    }

    #[inline]
    pub fn guard() {
        if IsShutDown::get() {
            trap("The canister has been halted until the next code upgrade.")
        }
    }
}

#[update]
fn halt() {
    if caller() != Controller::get_principal() {
        trap("Only the controller can call this method.");
    }

    storage::get_mut::<IsShutDown>().0 = true;
}

#[update]
fn finish_pending_tasks(limit: u32) {
    if caller() != Controller::get_principal() {
        trap("Only the controller can call this method.");
    }

    for _ in 0..limit {
        if !crate::progress().await {
            return;
        }
    }
}