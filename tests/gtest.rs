use gstd::ActorId;
use gstd::str::FromStr;
use sails_rs::{calls::*, CodeId, gtest::calls::*, U256};
use nexus_dao_client::ProposalStatus;
use vft_service::Service as VftService;

use nexus_dao_client::traits::*;

const ACTOR_ID: u64 = 42;
const NEW_ADMIN: u64 = 43;

#[tokio::test]
async fn create_and_query_multiple_daos() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());

    let program_id = program_factory
        // .new(CodeId::from_str("0x7ecca7e47b3e73a4cb0c1c6d4788ed442891dfd468dd29a59ce72c54c3c01902").unwrap() ) // Call program's constructor
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());


    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "TOKEN1".to_string(), "T1".to_string(), [].to_vec()).await;
    // Create multiple DAOs
    let _ = service_client
        .create_dao("DAO1".into(), "First DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "TOKEN2".to_string(), "T2".to_string(), [].to_vec()).await;
    let _ = service_client
        .create_dao("DAO2".into(), "Second DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    // Verify the first DAO's info
    let dao_info_1 = service_client
        .get_dao_info("DAO1".to_string())
        .recv(program_id)
        .await
        .unwrap().unwrap();

    assert_eq!(dao_info_1.name, "DAO1".to_string());
    assert_eq!(dao_info_1.description, "First DAO".to_string());
    assert_eq!(dao_info_1.token.name, "TOKEN1".to_string());
    assert_eq!(dao_info_1.token.symbol, "T1".to_string());


    // Verify the second DAO's info
    let dao_info_2 = service_client
        .get_dao_info("DAO2".to_string())
        .recv(program_id)
        .await
        .unwrap().unwrap();

    assert_eq!(dao_info_2.name, "DAO2".to_string());
    assert_eq!(dao_info_2.description, "Second DAO".to_string());
    assert_eq!(dao_info_2.token.name, "TOKEN2".to_string());
    assert_eq!(dao_info_2.token.symbol, "T2".to_string());

    let daos = service_client
        .get_daos_by_actor(ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(daos, vec!["DAO1".to_string(), "DAO2".to_string()]);
}

#[tokio::test]
async fn add_admin_and_check_admin() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());

    let program_id = program_factory
        // .new(CodeId::from_str("0x7ecca7e47b3e73a4cb0c1c6d4788ed442891dfd468dd29a59ce72c54c3c01902").unwrap()  )
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());

    let nexus_vft_id = get_vft_id(remoting, ACTOR_ID.into(), "Token".to_string(), "Symbol".to_string(), [].to_vec()).await;
    // Create a DAO
    let _ = service_client
        .create_dao("TestDAO".into(), "A test DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    // Add a new admin
    let _ = service_client
        .add_admin("TestDAO".into(), NEW_ADMIN.into())
        .send_recv(program_id)
        .await
        .unwrap();

    // Verify the new admin
    let is_admin = service_client
        .is_admin("TestDAO".into(), NEW_ADMIN.into())
        .recv(program_id)
        .await
        .unwrap();

    assert!(is_admin);

    // Check if the original creator is still an admin
    let is_creator_admin = service_client
        .is_admin("TestDAO".into(), ACTOR_ID.into())
        .recv(program_id)
        .await
        .unwrap();

    assert!(is_creator_admin);
}

#[tokio::test]
async fn test_dao_creation() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    // Submit program code into the system
    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());
    let program_id = program_factory
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());

    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "DAO Token".to_string(), "DT".to_string(), [].to_vec()).await;
    service_client
        .create_dao("TestDAO".into(), "A test DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    let dao_info = service_client
        .get_dao_info("TestDAO".to_string())
        .recv(program_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(dao_info.name, "TestDAO".to_string());
    assert_eq!(dao_info.description, "A test DAO".to_string());
    assert_eq!(dao_info.token.name, "DAO Token".to_string());
    assert_eq!(dao_info.token.symbol, "DT".to_string());
}

#[tokio::test]
async fn test_proposal_creation() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());
    let program_id = program_factory
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());

    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "DAO Token".to_string(), "DT".to_string(), [].to_vec()).await;
    service_client
        .create_dao("TestDAO".into(), "A test DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    service_client
        .create_proposal("TestDAO".into(), "Proposal 1".into(), "Detail 1".into(), 10, 20)
        .send_recv(program_id)
        .await
        .unwrap();

    let proposals = service_client
        .get_proposals("TestDAO".to_string())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].description, "Detail 1");

    // Call get_proposal to verify the proposal by id
    let proposal = service_client
        .get_proposal("TestDAO".to_string(), 1)
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(proposal.unwrap().description, "Detail 1");

}

#[tokio::test]
async fn test_proposal_voting() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());
    let program_id = program_factory
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());

    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "DAO Token".to_string(), "DT".to_string(), [].to_vec()).await;
    service_client
        .create_dao("TestDAO".into(), "A test DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    let proposal_id = service_client
        .create_proposal("TestDAO".into(), "Proposal 1".into(), "Detail 1".into(), 10, 20)
        .send_recv(program_id)
        .await
        .unwrap();

    let vote_result = service_client
        .vote_on_proposal("TestDAO".into(), proposal_id, true)
        .send_recv(program_id)
        .await.unwrap();;

    println!("res A  = {:?}", vote_result);
    assert!(vote_result.is_err());

    remoting.system().spend_blocks(11);
    let vote_result = service_client
        .vote_on_proposal("TestDAO".into(), proposal_id, true)
        .send_recv(program_id)
        .await.unwrap();;

    assert!(vote_result.is_ok());

    let proposals = service_client
        .get_proposals("TestDAO".to_string())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(proposals[0].votes_for, 1);

    let vote_result = service_client
        .vote_on_proposal("TestDAO".into(), proposal_id, false)
        .send_recv(program_id)
        .await.unwrap();;

    assert!(vote_result.is_ok());

    let proposals = service_client
        .get_proposals("TestDAO".to_string())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(proposals[0].votes_against, 1);
}

#[tokio::test]
async fn test_proposal_finalization() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());
    let program_id = program_factory
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());

    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "DAO Token".to_string(), "DT".to_string(), [].to_vec()).await;
    service_client
        .create_dao("TestDAO".into(), "A test DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    let proposal_id = service_client
        .create_proposal("TestDAO".into(), "Proposal 1".into(), "Detail 1".into(), 10, 20)
        .send_recv(program_id)
        .await
        .unwrap();

    remoting.system().spend_blocks(15);

    let _ = service_client
        .vote_on_proposal("TestDAO".into(), proposal_id, true)
        .send_recv(program_id)
        .await.unwrap();;

    // Simulate passing the voting period
    remoting.system().spend_blocks(21);

    let _ = service_client
        .finalize_proposal("TestDAO".into(), proposal_id)
        .send_recv(program_id)
        .await
        .unwrap();

    let proposals = service_client
        .get_proposals("TestDAO".to_string())
        .recv(program_id)
        .await
        .unwrap();

    assert_eq!(proposals[0].status, ProposalStatus::Passed);
}

#[tokio::test]
async fn test_vote_on_nonexistent_proposal() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());
    let program_id = program_factory
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());

    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "DAO Token".to_string(), "DT".to_string(), [].to_vec()).await;
    service_client
        .create_dao("TestDAO".into(), "A test DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    let result = service_client
        .vote_on_proposal("TestDAO".into(), 999, true)
        .send_recv(program_id)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_finalize_proposal_before_voting_end() {
    let remoting = GTestRemoting::new(ACTOR_ID.into());
    remoting.system().init_logger();

    let program_code_id = remoting.system().submit_code(nexus_dao::WASM_BINARY);

    let program_factory = nexus_dao_client::NexusDaoFactory::new(remoting.clone());
    let program_id = program_factory
        .new()
        .send_recv(program_code_id, b"salt")
        .await
        .unwrap();

    let mut service_client = nexus_dao_client::NexusDao::new(remoting.clone());

    let nexus_vft_id = get_vft_id(remoting.clone(), ACTOR_ID.into(), "DAO Token".to_string(), "DT".to_string(), [].to_vec()).await;
    service_client
        .create_dao("TestDAO".into(), "A test DAO".into(), nexus_vft_id)
        .send_recv(program_id)
        .await
        .unwrap();

    let proposal_id = service_client
        .create_proposal("TestDAO".into(), "Proposal 1".into(), "Detail 1".into(), 10, 20)
        .send_recv(program_id)
        .await
        .unwrap();

    remoting.system().spend_blocks(5);

    let result = service_client
        .finalize_proposal("TestDAO".into(), proposal_id)
        .send_recv(program_id)
        .await
        .unwrap();

    assert!(result.is_err());
}

use nexus_vft_client::traits::*;

async fn get_vft_id(remoting: GTestRemoting, actor_id: ActorId, name: String, symbol: String, initial_balance: Vec<(ActorId, U256)>) -> ActorId {

    // let program_code_id = remoting.system().submit_code(nexus_vft_client::NexusVft::WASM_BINARY);
    let program_code_id = remoting.system().submit_code_file("./nexus_vft.opt.wasm");

    let nexus_vft_factory = nexus_vft_client::NexusVftFactory::new(remoting.clone());
    let nexus_vft_id = nexus_vft_factory
        .initialize(name.clone(), symbol, 18, initial_balance)
        .send_recv(program_code_id, name.as_bytes())
        .await
        .unwrap();

    nexus_vft_id

}

