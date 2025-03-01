use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;

use crate::handler::{submit_packet, to_error_packet};
use crate::orm::model::{account, persona, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_errors::EAError;
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rq_user(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    let lkey = prq.packet.data.get("LKEY").unwrap();
    let tid = prq.packet.data.get("TID").unwrap();
    // Other, unused fields
    // let cid = prq.packet.data.get("CID"); // Usually empty
    // let mac = prq.packet.data.get("MAC"); // Usually some hash
    // let sku = prq.packet.data.get("SKU"); // Usually 'pc'
    // let name = prq.packet.data.get("NAME"); // Usually empty

    // Get the user via the lobby key
    let Ok(Some(db_account)) = account::Entity::find()
        .filter(account::Column::LobbyKey.eq(lkey.clone()))
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(
            &prq.packet,
            EAError::EA_AuthFail as i32,
            Some("Account not found".to_string()),
        );
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Account not found.");
    };

    // Get the session vai the lobby key
    let Ok(Some(db_session)) = session::Entity::find()
        .filter(session::Column::LobbyKey.eq(lkey.clone()))
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(
            &prq.packet,
            EAError::EA_AuthFail as i32,
            Some("Account not found".to_string()),
        );
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Session not initialed via FESL.");
    };

    // Extract account data into session values
    let user_id = db_account.id;
    let user_name = db_account.email.to_string();
    let persona_id = db_session.persona_id;

    assert_eq!(user_id, db_session.user_id);

    // Find persona for ID
    let Ok(Some(db_persona)) = persona::Entity::find_by_id(persona_id)
        .one(&*prq.sstate.database)
        .await
    else {
        let err_pkt = to_error_packet(
            &prq.packet,
            EAError::EA_AuthFail as i32,
            Some("Account not found".to_string()),
        );
        submit_packet(err_pkt, &prq.con, &prq.sstate, 0).await;
        return Err("Persona for given Persona ID not found.");
    };

    // Add theater handle to session
    let mut active_session = db_session.into_active_model();
    active_session.theater_tcp_handle = Set(prq.con.to_string());
    let _ = active_session.update(&*prq.sstate.database).await;

    let mut response_hm: IndexMap<String, String> = IndexMap::new();
    //response_hm.insert("NAME".to_string(), user_name.to_string());
    response_hm.insert("NAME".to_string(), db_persona.name.to_string());
    response_hm.insert("TID".to_string(), tid.to_string());

    let response_packet = DataPacket {
        packet_mode: PacketMode::FeslPingOrTheaterResponse,
        mode: DataMode::THEATER_USER,
        packet_id: prq.packet.packet_id,
        data: response_hm,
    };

    // Enqueue the response
    submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;

    Ok(())
}
