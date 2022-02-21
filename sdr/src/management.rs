use ic_kit::macros::*;
use ic_kit::{get_context, Context, Principal};

// --- init

pub struct Controller(Option<Principal>);

impl Default for Controller {
    fn default() -> Self {
        Controller(None)
    }
}

impl Controller {
    pub fn load(controller: Principal) {
        let ic = get_context();
        let data = ic.get_mut::<Controller>();
        data.0 = Some(controller);
    }

    pub fn load_if_not_present(controller: Principal) {
        let ic = get_context();
        let data = ic.get_mut::<Controller>();
        if data.0.is_none() {
            data.0 = Some(controller);
        }
    }

    #[inline]
    pub fn get_principal() -> Principal {
        let ic = get_context();
        ic.get::<Controller>().0.unwrap().clone()
    }
}

#[init]
fn init() {
    let ic = get_context();
    Controller::load_if_not_present(ic.caller());
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
        let ic = get_context();
        ic.get::<IsShutDown>().0
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
    let ic = get_context();

    if ic.caller() != Controller::get_principal() {
        panic!("Only the controller can call this method.");
    }

    ic.get_mut::<IsShutDown>().0 = true;
}

#[update]
async fn finish_pending_tasks(limit: u32) {
    let ic = get_context();

    if ic.caller() != Controller::get_principal() {
        panic!("Only the controller can call this method.");
    }

    for _ in 0..limit {
        if !crate::progress().await {
            return;
        }
    }
}
