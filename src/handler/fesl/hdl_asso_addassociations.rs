use indexmap::IndexMap;

use crate::handler::{submit_packet, to_error_packet};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


pub const LOTRCQ_DOMAIN: &str = "eadm";
pub const LOTRCQ_SUBDOMAIN: &str = "eadm";
pub const LOTRCQ_PARTITION_ID: &str = "online_content";

pub async fn asso_addassociations(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // User should be authenticated
    if !prq.is_authenticated_user().await {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("User not authenticated.");
    }

    let Some(db_session) = prq.get_active_session_model().await else {
        panic!("Session not found although authenticated earlier...");
    };

    let Some(db_account) = prq.get_active_user_model().await else {
        panic!("User not found although authenticated earlier...");
    };

    // Read the request parameters
    let domainPartition_domain = prq
        .packet
        .data
        .get("domainPartition.domain")
        .unwrap()
        .to_string(); // "pc"; matches "domainPartition.domain" in the FSYS/Hello response
    let domainPartition_subDomain = prq
        .packet
        .data
        .get("domainPartition.subDomain")
        .unwrap()
        .to_string(); // "LOTR"; matches "domainPartition.subDomain" in the FSYS/Hello response
    let domainPartition_key = prq
        .packet
        .data
        .get("domainPartition.key")
        .unwrap()
        .to_string(); // ""; seems to be empty; to be set in the response
    let assoType = match prq.packet.data.get("type") {
        Some(assoType) => assoType.to_string(),
        None => "".to_string(),
    }; // "PlasmaMute", "PlasmaBlock", "PlasmaRecentPlayers", "PlasmaFriends"

    // owner.id matches the user_id
    let owner_id = prq.packet.data.get("owner.id").unwrap().to_string(); // "1" -> Identical to nuid/user id, if not further separated
    // Seems to be 1 for a normal user
    let owner_type = prq.packet.data.get("owner.type").unwrap().to_string(); // "1"

    // Check if the owner.id matches the user_id
    if &owner_id != &db_session.user_id.to_string() {
        let err_pkt = to_error_packet(&prq.packet, EAError::EA_AuthFail as i32, None);
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Invalid owner.id");
    }

    // What to do if the list is full...
    let list_full_behavior = prq.packet.data.get("listFullBehavior").cloned().unwrap_or("".to_string());

    // Read how many request items are in the packet
    let add_requests_count = prq.packet.data.get("addRequests.[]").cloned().unwrap_or("0".to_string()).parse::<u32>().unwrap_or(0);

    let max_list_size: u32 = if assoType == "PlasmaRecentPlayers" {20} else {100};

    let mut response_hm: IndexMap<_, _, _> = IndexMap::new();
    response_hm.insert("TXN".to_string(), "AddAssociations".to_string());
    // Domain partition
    response_hm.insert(
        "domainPartition.domain".to_string(),
        LOTRCQ_DOMAIN.to_string(),
    );
    response_hm.insert(
        "domainPartition.subDomain".to_string(),
        domainPartition_key.to_string(),
    );
    response_hm.insert(
        "domainPartition.key".to_string(),
        domainPartition_key.to_string(),
    ); // Optional?!

    response_hm.insert("type".to_string(), assoType.to_string());
    response_hm.insert("maxListSize".to_string(), max_list_size.to_string());

    response_hm.insert("result.[]".to_string(), "0".to_string());

    let response = DataPacket::new(
        DataMode::FESL_ASSO,
        PacketMode::FeslSinglePacketResponse,
        prq.packet.packet_id,
        response_hm,
    );

    submit_packet(response, &prq.con, &prq.sstate, 0).await;
    Ok(())
}
