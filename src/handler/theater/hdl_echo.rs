use indexmap::IndexMap;
use sea_orm::entity::*;
use sea_orm::query::*;
use tracing::info;

use crate::client_connection::{ClientConnectionDescriptor, ProtoType};
use crate::handler::submit_packet;
use crate::orm::model::{game, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


pub async fn handle_rsp_echo(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // ECHO response is responsible for determining the NAT/firewall type and obtain the external IP and port of the client.

    // Ensure that it is in fact an UDP connection
    if prq.con.proto_type != ProtoType::Udp {
        return Err("ECHO response received on non-UDP connection");
    }

    // Get user id (=account id) from the packet
    let uid = prq.packet.data.get("UID").unwrap();
    let echo_type = prq.packet.data.get("TYPE").unwrap(); // Should be const value "1"
    let tid = prq.packet.data.get("TID").unwrap();

    // Parse UID
    let Ok(uid_int) = uid.parse::<i64>() else {
        return Err("UID not parsable");
    };

    // Find related session
    let Ok(Some(db_session)) = session::Entity::find()
        .filter(session::Column::UserId.eq(uid_int))
        .one(&*prq.sstate.database)
        .await
    else {
        return Err("Session not found");
    };

    // Check the current nat findings about the session
    // We know the following "types":
    // NAT_UNKNOWN: 0, NAT_OPEN: 1, NAT_SIMPLE: 2, NAT_STRICT: 3
    let current_nat_type = db_session.nat_type;
    let udp_port_changed: bool;

    // Check if the session has a udp handle set and if it matches the current connection
    if db_session.theater_udp_handle == "" {
        // We don't have any information on the udp handle.
        // -> This is the first time we set the udp handle!
        let mut db_session: session::ActiveModel = db_session.clone().into_active_model();
        db_session.theater_udp_handle = Set(prq.con.to_string());
        let Ok(_) = db_session.update(&*prq.sstate.database).await else {
            return Err("Failed to update session");
        };
        udp_port_changed = false;
    } else if db_session.theater_udp_handle != prq.con.to_string() {
        let old_handle = db_session.theater_udp_handle.clone();
        // The UDP handle does not match the current connection
        // -> Update the UDP handle
        let mut db_session: session::ActiveModel = db_session.clone().into_active_model();
        db_session.theater_udp_handle = Set(prq.con.to_string());
        let Ok(_) = db_session.update(&*prq.sstate.database).await else {
            return Err("Failed to update session");
        };
        udp_port_changed = true;
        info!(
            target: "nat",
            "UDP handle mismatch: Old handle: {}, New handle: {}",
            old_handle, prq.con.to_string()
        );
    } else {
        // The UDP handle matches the current connection
        udp_port_changed = false;
    }

    // Get external UDP information
    let external_ip = &prq.con.client_ip;
    let external_port = prq.con.client_port;

    // Prepare the response
    let mut response_hm = IndexMap::new();
    response_hm.insert(
        "TID".to_string(),
        prq.packet.data.get("TID").unwrap().to_string(),
    );
    response_hm.insert("IP".to_string(), external_ip.to_string());
    response_hm.insert("PORT".to_string(), external_port.to_string());
    response_hm.insert("ERR".to_string(), "0".to_string());
    response_hm.insert("TYPE".to_string(), echo_type.to_string());

    // Check if it is the echo phase during login or during game creation / joining
    if prq.packet.data.contains_key("UGID") && prq.packet.data.contains_key("SECRET") {
        // The player has created a game -> Hence, we can actually differentiate
        // between NAT_SIMPLE and NAT_STRICT here because we can check the advertised game port.

        if db_session.nat_type != 1 {
            // The NAT type is not NAT_OPEN -> We need to further differentiate the NAT type
            let persona_id = db_session.persona_id;
            // Get the game from the database
            let Ok(Some(db_game)) = game::Entity::find()
                .filter(game::Column::PersonaId.eq(persona_id))
                .one(&*prq.sstate.database)
                .await
            else {
                return Err("Game not found");
            };
            if !udp_port_changed
                && db_game.port == db_game.internal_port
                && db_game.port == external_port as i32
            {
                // The external port has not changed and is identical to the advertised game port
                // -> We assume NAT_SIMPLE
                let mut db_session: session::ActiveModel = db_session.clone().into_active_model();
                db_session.nat_type = Set(2);
                let Ok(_) = db_session.update(&*prq.sstate.database).await else {
                    return Err("Failed to update session");
                };
            } else {
                // The external port has changed or is not identical to the advertised game port
                // -> We keep assuming NAT_STRICT
            }
        }

        // Send the response
        let response_packet: DataPacket = DataPacket {
            packet_mode: PacketMode::FeslPingOrTheaterResponse,
            mode: DataMode::THEATER_ECHO,
            packet_id: 0,
            data: response_hm,
        };
        submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;
        return Ok(());
    }

    // We now try to determine the NAT type.
    // The NAT type is determined by the following rules:
    //
    // - The first response probes NAT_OPEN.
    //   If STUNRelay is enabled, we can try the following:
    //     - Send the response via the remote STUNRelay (with other port and IP address).
    //       - If the client receives this, it is NAT_OPEN.
    //   If STUNRelay is NOT enabled, we mitigate it as follows:
    //     - Send the response from this server, but on a different port.
    //       - If the client receives this, it is NAT_OPEN.
    //   If the client does not receive the response, it will re-send the echo request.
    //
    // - The second response automatically implies NAT_SIMPLE or NAT_STRICT.
    //   The type is assumed to be NAT_SIMPLE if all of the following conditions are met:
    //     - The external port of the UDP connection has not changed.
    //     - The external port is identical to the advertised game port. (<- We can only check this at the CGAM or EGAM stage)
    //     - The client won't immediately send an additional echo request.
    //   Otherwise, it is NAT_STRICT.
    //   Since we cannot check the advertised game port, we assume NAT_STRICT here (until we hit EGAM or CGAM).
    //   If the client does not receive this, it will re-send the echo request.
    //
    // - The third time the client sends the echo request, the server will assume that the client is behind a NAT_STRICT.

    match current_nat_type {
        0 => {
            // This is the first echo packet -> we assume NAT_OPEN
            let stun_con: ClientConnectionDescriptor;
            if prq.sstate.stunrelay.enabled {
                // We respond on the remote STUNRelay
                let mut stunrelay_con = prq.con.clone();
                stunrelay_con.proto_type = ProtoType::RemoteUdp;
                stunrelay_con.host_port = prq.sstate.stunrelay.relay_source_port;
                stun_con = stunrelay_con;
            } else {
                // We respond on the same connection, but with a different port.
                let mut localstun_con = prq.con.clone();
                localstun_con.host_port = prq.sstate.stunrelay.relay_source_port;
                stun_con = localstun_con;
            }

            // Set the NAT type to NAT_OPEN
            let mut db_session: session::ActiveModel = db_session.into_active_model();
            db_session.nat_type = Set(1);
            let Ok(_) = db_session.update(&*prq.sstate.database).await else {
                return Err("Failed to update session");
            };

            // Send the packet via STUNRelay or local port
            let response_packet: DataPacket = DataPacket {
                packet_mode: PacketMode::FeslPingOrTheaterResponse,
                mode: DataMode::THEATER_ECHO,
                packet_id: 0,
                data: response_hm,
            };

            submit_packet(response_packet, &stun_con, &prq.sstate, 0).await;
        }
        1 => {
            // This is the second echo packet -> we assume NAT_SIMPLE or NAT_STRICT
            // But we only see the advertised port at CGAM or EGAM, so we can't really determine the NAT type here.
            // Hence, we will assume NAT_STRICT here.

            // Update the session with the new NAT type
            let mut db_session: session::ActiveModel = db_session.clone().into_active_model();
            db_session.nat_type = Set(3);
            let Ok(_) = db_session.update(&*prq.sstate.database).await else {
                return Err("Failed to update session");
            };

            // Send the packet via various methods
            let response_packet: DataPacket = DataPacket {
                packet_mode: PacketMode::FeslPingOrTheaterResponse,
                mode: DataMode::THEATER_ECHO,
                packet_id: 0,
                data: response_hm,
            };

            submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;
        }
        _ => {
            // Just send the response otherwise (probably NAT_STRICT)
            let response_packet: DataPacket = DataPacket {
                packet_mode: PacketMode::FeslPingOrTheaterResponse,
                mode: DataMode::THEATER_ECHO,
                packet_id: 0,
                data: response_hm,
            };
            submit_packet(response_packet, &prq.con, &prq.sstate, 0).await;
        }
    }

    Ok(())
}
