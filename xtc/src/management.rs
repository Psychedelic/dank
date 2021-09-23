use ic_kit::Principal;
use ic_kit::{ic, macros::*};

// --- init

#[derive(Default)]
pub struct Controller(Option<Principal>);

impl Controller {
    pub fn load(controller: Principal) {
        let data = ic::get_mut::<Controller>();
        data.0 = Some(controller);
    }

    pub fn load_if_not_present(controller: Principal) {
        let data = ic::get_mut::<Controller>();
        if data.0.is_none() {
            data.0 = Some(controller);
        }
    }

    #[inline]
    pub fn get_principal() -> Principal {
        ic::get::<Controller>().0.unwrap()
    }
}

#[init]
fn init() {
    Controller::load_if_not_present(ic::caller());
}

// --- halt

#[derive(Default)]
pub struct IsShutDown(bool);

impl IsShutDown {
    #[inline]
    pub fn get() -> bool {
        ic::get::<IsShutDown>().0
    }

    #[inline]
    pub fn guard() {
        if IsShutDown::get() {
            panic!("The canister has been halted until the next code upgrade.")
        }
    }
}

#[update]
fn halt() {
    if ic::caller() != Controller::get_principal() {
        panic!("Only the controller can call this method.");
    }

    ic::get_mut::<IsShutDown>().0 = true;
}

#[update]
async fn finish_pending_tasks(limit: u32) {
    if ic::caller() != Controller::get_principal() {
        panic!("Only the controller can call this method.");
    }

    for _ in 0..limit {
        if !crate::progress().await {
            return;
        }
    }
}
