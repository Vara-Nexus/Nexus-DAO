#![no_std]

mod storage_dao_collection;
mod storage_dao_map;
mod storage_proposal_map;

use gstd::{debug, prog};

#[cfg(feature = "wasm-binary")]
#[cfg(not(target_arch = "wasm32"))]
pub use code::WASM_BINARY_OPT as WASM_BINARY;
use gstd::collections::HashMap;
use gstd::{exec, msg};
use sails_rs::prelude::*;
use crate::storage_dao_collection::{DaoCollection, DAO_COLLECTION, DaoState};
use crate::storage_dao_map::{ACTOR_DAO_MAP, ActorDaoMap};
use crate::storage_proposal_map::{PROPOSAL_MAP, Proposal, ProposalMap, ProposalStatus};


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Event {
    DaoCreated {
        name: String,
        creator: ActorId,
        token_actor: ActorId,
        creation_block: u64
    },
    AdminAdded { admin: ActorId },
    ProposalCreated {
        dao_name: String,
        proposal_id: u32,
        creator: ActorId,
    },
    ProposalVoted {
        dao_name: String,
        proposal_id: u32,
        voter: ActorId,
        vote_for: bool,
    },
    ProposalFinalized {
        dao_name: String,
        proposal_id: u32,
        status: ProposalStatus,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum Error {
    NotInVotingPeriod,
    VoteNotEnded,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = sails_rs::scale_codec)]
#[scale_info(crate = sails_rs::scale_info)]
struct ResultDaoInfo<T> {
    name: String,
    description: String,
    token_actor: ActorId,
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

#[sails_rs::service(events = Event)]
impl NexusDaoService {
    pub fn new() -> Self {
        Self(())
    }

    pub fn init() -> Self {
        unsafe {

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
            if PROPOSAL_MAP.is_none() {
                PROPOSAL_MAP = Some(ProposalMap {
                    dao_to_proposals: Default::default(),
                });
            }
        }
        Self(())
    }

    pub fn create_dao(&mut self, name: String, description: String, token_actor: ActorId) -> bool {
        let creator = msg::source();
        let creation_block = exec::block_height().into();

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
        actor_map.actor_to_daos.entry(creator).or_insert_with(Vec::new).push(name.clone());

        // Notify about DAO creation
        let _ = self.notify_on(Event::DaoCreated {
            name,
            creator,
            token_actor,
            creation_block
        });

        true
    }

    pub fn create_proposal(&mut self, dao_name: String, title: String, description: String, voting_start: u32, voting_end: u32) -> u32 {
        let creator = msg::source();
        let proposals = ProposalMap::get_mut().dao_to_proposals.entry(dao_name.clone()).or_insert_with(Vec::new);
        let proposal_id = proposals.len() as u32 + 1;

        if voting_start >= voting_end {
            panic!("Voting start must be before voting end")
        }

        proposals.push(Proposal {
            title,
            description,
            creator,
            voting_start,
            voting_end,
            status: ProposalStatus::Active,
            votes_for: 0,
            votes_against: 0,
        });

        let _ = self.notify_on(Event::ProposalCreated {
            dao_name,
            proposal_id: proposal_id.clone(),
            creator,
        });

        proposal_id
    }

    pub fn vote_on_proposal(&mut self, dao_name: String, proposal_id: u32, vote_for: bool) -> Result<(), Error> {
        let voter = msg::source();
        let proposals = ProposalMap::get_mut().dao_to_proposals.get_mut(&dao_name).expect("DAO not found");
        let proposal = proposals.get_mut(proposal_id as usize - 1).expect("Proposal not found");

        if exec::block_height() < proposal.voting_start || exec::block_height() > proposal.voting_end {
            return Err(Error::NotInVotingPeriod);
        }

        if vote_for {
            proposal.votes_for += 1;
        } else {
            proposal.votes_against += 1;
        }

        let _ = self.notify_on(Event::ProposalVoted {
            dao_name,
            proposal_id,
            voter,
            vote_for,
        });

        Ok(())
    }

    pub fn finalize_proposal(&mut self, dao_name: String, proposal_id: u32) -> Result<(), Error> {

        let proposals = ProposalMap::get_mut().dao_to_proposals.get_mut(&dao_name).expect("DAO not found");
        let proposal = proposals.get_mut(proposal_id as usize - 1).expect("Proposal not found");

        if exec::block_height() < proposal.voting_end {
            return Err(Error::VoteNotEnded)
        }

        if proposal.votes_for > proposal.votes_against {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        let _ = self.notify_on(Event::ProposalFinalized {
            dao_name,
            proposal_id,
            status: proposal.status.clone(),
        });

        Ok(())
    }

    pub fn get_proposals(&self, dao_name: String) -> Vec<Proposal> {
        ProposalMap::get().dao_to_proposals.get(&dao_name).cloned().unwrap_or_default()
    }

    pub async fn get_all_dao_info(&self) -> Vec<ResultDaoInfo<()>> {
        let state = DaoCollection::get();
        let mut result = Vec::new();

        for (name, dao) in state.daos.iter() {
            result.push(ResultDaoInfo {
                name: name.clone(),
                description: dao.description.clone(),
                token_actor: dao.token,
                token: (),
            });
        }
        result
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
                name: name,
                description: dao.description.clone(),
                token_actor: dao.token,
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

                // Notify about the admin being added
                let _ = self.notify_on(Event::AdminAdded { admin: new_admin });
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
