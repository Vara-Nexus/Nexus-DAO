#![no_std]

use gstd::ActorId;
use gstd::collections::HashMap;
use sails_rs::prelude::*;

pub(crate) static mut ACTOR_DAO_MAP: Option<ActorDaoMap> = None;

pub struct ActorDaoMap {
    pub(crate) actor_to_daos: HashMap<ActorId, Vec<String>>,
}

impl ActorDaoMap {
    pub fn get() -> &'static Self {
        unsafe { ACTOR_DAO_MAP.as_ref().expect("ActorDaoMap is not initialized") }
    }

    pub fn get_mut() -> &'static mut Self {
        unsafe { ACTOR_DAO_MAP.as_mut().expect("ActorDaoMap is not initialized") }
    }
}

