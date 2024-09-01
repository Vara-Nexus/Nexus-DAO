#![no_std]

use gstd::ActorId;
use gstd::collections::HashMap;
use sails_rs::prelude::*;

pub(crate) static mut PROPOSAL_MAP: Option<ProposalMap> = None;

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct Proposal {
    pub description: String,
    pub creator: ActorId,
    pub voting_start: u32,
    pub voting_end: u32,
    pub status: ProposalStatus,
    pub votes_for: u32,
    pub votes_against: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, TypeInfo)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
}

pub struct ProposalMap {
    pub(crate) dao_to_proposals: HashMap<String, Vec<Proposal>>,
}

impl ProposalMap {
    pub fn get() -> &'static Self {
        unsafe { PROPOSAL_MAP.as_ref().expect("ProposalMap is not initialized") }
    }

    pub fn get_mut() -> &'static mut Self {
        unsafe { PROPOSAL_MAP.as_mut().expect("ProposalMap is not initialized") }
    }

}
