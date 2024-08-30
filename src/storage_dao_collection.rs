#![no_std]

use gstd::collections::HashMap;
use gstd::{exec, msg};
use sails_rs::prelude::*;
use vft_service::Service as VftService; // Import VFT standard service

pub(crate) static mut DAO_COLLECTION: Option<DaoCollection> = None;

pub struct DaoState {
    pub description: String,
    // pub token: VftService,
    pub token: ActorId, // Use ActorId instead of VftService to represent the token contract
    pub admins: Vec<ActorId>,
    pub creator: ActorId,
    pub creation_block: u64,
}

pub struct DaoCollection {
    pub daos: HashMap<String, DaoState>,
}

impl DaoCollection {
    pub fn get() -> &'static Self {
        unsafe { DAO_COLLECTION.as_ref().expect("DAO collection is not initialized") }
    }

    pub fn get_mut() -> &'static mut Self {
        unsafe { DAO_COLLECTION.as_mut().expect("DAO collection is not initialized") }
    }
}
