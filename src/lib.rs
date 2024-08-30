#![no_std]

mod storage_dao_collection;
mod storage_dao_map;
use gstd::{debug, prog};

#[cfg(feature = "wasm-binary")]
#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;
use gstd::collections::HashMap;
use gstd::{exec, msg};
use sails_rs::prelude::*;
use crate::storage_dao_collection::{DaoCollection, DAO_COLLECTION, DaoState};
use crate::storage_dao_map::{ACTOR_DAO_MAP, ActorDaoMap};

static mut DAO_BASE_VFT_CODE: Option<CodeId> = None;

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
struct ResultDaoInfo<T> {
    name: String,
    description: String,
    token: T,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
struct ResultTokenInfo {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: U256,
}

struct NexusDaoService(());

#[sails_rs::service]
impl NexusDaoService {
    pub fn new() -> Self {
        Self(())
    }

    pub fn init() -> Self {
        unsafe {
            // DAO_BASE_VFT_CODE = Some(vft_code);

            if DAO_COLLECTION.is_none() {
                DAO_COLLECTION = Some(DaoCollection {
                    daos: HashMap::new(),
                });
            }
            if ACTOR_DAO_MAP.is_none() {
                ACTOR_DAO_MAP = Some(ActorDaoMap {
                    actor_to_daos: HashMap::new(),
                });
            }
        }
        Self(())
    }

    pub fn create_dao(&mut self, name: String, description: String, token_actor: ActorId) {
        let creator = msg::source();
        let creation_block = exec::block_height().into();

        // let code_id: CodeId = msg::load().expect("Unable to load");
        // debug!("CodeId: {:?}", code_id);
        // let (init_message_id, new_program_id) =
        //     prog::create_program_bytes(code_id, "salt2", b"New", 0)
        //         .expect("Unable to create a program");
        //
        // debug!("New program id: {:?}", new_program_id);
        // msg::send_bytes(new_program_id, b"PING", 0).expect("Unable to send");

        let state = DaoCollection::get_mut();
        state.daos.insert(
            name.clone(),
            DaoState {
                description,
                token: token_actor,
                admins: vec![creator],
                creator,            // Set the creator
                creation_block,     // Set the creation block number
            },
        );

        let actor_map = ActorDaoMap::get_mut();
        actor_map.actor_to_daos.entry(creator).or_insert_with(Vec::new).push(name);
    }

    pub async fn get_dao_info(&self, name: String) -> Option<ResultDaoInfo<ResultTokenInfo>> {
        let state = DaoCollection::get();

        if let Some(dao) = state.daos.get(&name) {
            let token_info = {
                // Query the token name
                let call_payload = nexus_vft_client::nexus_vft::io::Name::encode_call();
                let reply_bytes = gstd::msg::send_bytes_for_reply(dao.token, call_payload, 0, 0)
                    .unwrap()
                    .await
                    .unwrap();
                let token_name = <nexus_vft_client::nexus_vft::io::Name as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes).unwrap();

                // Query the token symbol
                let call_payload = nexus_vft_client::nexus_vft::io::Symbol::encode_call();
                let reply_bytes = gstd::msg::send_bytes_for_reply(dao.token, call_payload, 0, 0)
                    .unwrap()
                    .await
                    .unwrap();
                let token_symbol = <nexus_vft_client::nexus_vft::io::Symbol as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes).unwrap();

                // Query the token decimals
                let call_payload = nexus_vft_client::nexus_vft::io::Decimals::encode_call();
                let reply_bytes = gstd::msg::send_bytes_for_reply(dao.token, call_payload, 0, 0)
                    .unwrap()
                    .await
                    .unwrap();
                let token_decimals = <nexus_vft_client::nexus_vft::io::Decimals as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes).unwrap();

                // Query the token total supply
                let call_payload = nexus_vft_client::nexus_vft::io::TotalSupply::encode_call();
                let reply_bytes = gstd::msg::send_bytes_for_reply(dao.token, call_payload, 0, 0)
                    .unwrap()
                    .await
                    .unwrap();
                let token_total_supply = <nexus_vft_client::nexus_vft::io::TotalSupply as sails_rs::calls::ActionIo>::decode_reply(&reply_bytes).unwrap();

                ResultTokenInfo {
                    name: token_name,
                    symbol: token_symbol,
                    decimals: token_decimals,
                    total_supply: token_total_supply,
                }
            };

            Some(ResultDaoInfo {
                name: name.clone(),
                description: dao.description.clone(),
                token: token_info,
            })
        } else {
            None
        }
    }

    pub fn get_daos_by_actor(&self, actor: ActorId) -> Option<Vec<String>> {
        let actor_map = ActorDaoMap::get();
        actor_map.actor_to_daos.get(&actor).cloned()
    }

    pub fn add_admin(&mut self, dao_name: String, new_admin: ActorId) {
        let state = DaoCollection::get_mut();
        let dao = state.daos.get_mut(&dao_name).expect("DAO not found");

        let caller = msg::source();
        if dao.admins.contains(&caller) {
            if !dao.admins.contains(&new_admin) {
                dao.admins.push(new_admin);
            } else {
                panic!("The user is already an administrator");
            }
        } else {
            panic!("Only administrators can add new administrators");
        }
    }

    pub fn is_admin(&self, dao_name: String, user: ActorId) -> bool {
        let state = DaoCollection::get();
        let dao = state.daos.get(&dao_name).expect("DAO not found");
        dao.admins.contains(&user)
    }

}

pub struct NexusDaoProgram(());

#[sails_rs::program]
impl NexusDaoProgram {
    pub fn new() -> Self {
        NexusDaoService::init();
        Self(())
    }

    pub fn nexus_dao(&self) -> NexusDaoService {
        NexusDaoService::new()
    }
}

#[cfg(feature = "wasm-binary")]
#[cfg(not(target_arch = "wasm32"))]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}
