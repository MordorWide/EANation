use sea_orm::entity::*;
use sea_orm::query::*;
use std::sync::Arc;

use crate::client_connection::{ClientConnectionDescriptor, ProtoType, ServiceType};
use crate::handler::Handler;
use crate::orm::model::{game, participant, session};
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::sharedstate::SharedState;


const FESL_MEMCHECK_INTERVAL: u32 = 60; // 1 min
const FESL_PING_INTERVAL: u32 = 60; // 1 min

pub const LOTRCQ_PARTITION_ID: &str = "online_content";
pub const LOTRCQ_DOMAIN: &str = "eadm";
pub const LOTRCQ_SUBDOMAIN: &str = "eadm";
pub const LOTRCQ_GUID: &str = "lotr_conquest";
pub const LOTRCQ_CONTENTSTRING: &str = "lotr_conquest";
pub const LOBBY_ID: isize = 1;

mod hdl_fsys_hello;
use hdl_fsys_hello::fsys_hello;

mod hdl_fsys_goodbye;
use hdl_fsys_goodbye::fsys_goodbye;

mod hdl_fsys_getpingsites;
use hdl_fsys_getpingsites::fsys_getpingsites;

mod hdl_acct_getcountrylist;
use hdl_acct_getcountrylist::acct_getcountrylist;

mod hdl_acct_nugettos;
use hdl_acct_nugettos::acct_nugettos;

mod hdl_acct_nuaddaccount;
use hdl_acct_nuaddaccount::acct_nuaddaccount;

mod hdl_acct_nulogin;
use hdl_acct_nulogin::acct_nulogin;

mod hdl_acct_nuentitlegame;
use hdl_acct_nuentitlegame::acct_nuentitlegame;

mod hdl_acct_nugetpersona;
use hdl_acct_nugetpersona::acct_nugetpersona;

mod hdl_acct_nusuggestpersonas;
use hdl_acct_nusuggestpersonas::acct_nusuggestpersonas;

mod hdl_acct_nuloginpersona;
use hdl_acct_nuloginpersona::acct_nuloginpersona;

mod hdl_acct_nuaddpersona;
use hdl_acct_nuaddpersona::acct_nuaddpersona;

mod hdl_asso_getassociations;
use hdl_asso_getassociations::asso_getassociations;

mod hdl_asso_addassociations;
use hdl_asso_addassociations::asso_addassociations;

mod hdl_pres_setpresencestatus;
use hdl_pres_setpresencestatus::pres_setpresencestatus;

mod hdl_fsys_ping;
use hdl_fsys_ping::fsys_ping;

mod hdl_fsys_memcheck;
use hdl_fsys_memcheck::fsys_memcheck;

mod utils_memcheck;
use utils_memcheck::send_memcheck;

mod utils_ping;
use utils_ping::send_ping;

mod hdl_acct_nups3login;
use hdl_acct_nups3login::acct_nups3login;

mod hdl_rank_gettopnandme;
use hdl_rank_gettopnandme::rank_gettopnandme;

mod hdl_acct_nuxbl360login;
use hdl_acct_nuxbl360login::acct_nuxbl360login;

pub struct FeslHandler;

#[async_trait::async_trait]
impl Handler for FeslHandler {
    fn handler_type(&self) -> ServiceType {
        ServiceType::Fesl
    }

    async fn connection_closed(&self, con: ClientConnectionDescriptor, sstate: Arc<SharedState>) {
        // Check if the connection is a FESL connection
        if con.service_type != ServiceType::Fesl {
            return;
        }
        if con.proto_type != ProtoType::Tcp {
            return;
        }

        // Load session
        let Ok(db_sessions) = session::Entity::find()
            .filter(session::Column::FeslTcpHandle.eq(con.to_string()))
            .all(&*sstate.database)
            .await
        else {
            return;
        };

        for db_session in db_sessions {
            let persona_id = db_session.persona_id;

            // Check if the session has hosted games
            let Ok(db_games) = game::Entity::find()
                .filter(game::Column::PersonaId.eq(persona_id))
                .all(&*sstate.database)
                .await
            else {
                return;
            };

            for db_game in db_games {
                // Remove all participants of the game from the participant table...
                let Ok(db_participants) = participant::Entity::delete_many()
                    .filter(participant::Column::GameId.eq(db_game.id))
                    .exec(&*sstate.database)
                    .await
                else {
                    return;
                };
            }

            // Delete the game entries
            let Ok(_) = game::Entity::delete_many()
                .filter(game::Column::PersonaId.eq(persona_id))
                .exec(&*sstate.database)
                .await
            else {
                return;
            };

            // Clear the participant entries...
            let Ok(db_participants) = participant::Entity::delete_many()
                .filter(participant::Column::PersonaId.eq(persona_id))
                .exec(&*sstate.database)
                .await
            else {
                return;
            };
        }

        // Delete all sessions of the client
        let Ok(_) = session::Entity::delete_many()
            .filter(session::Column::FeslTcpHandle.eq(con.to_string()))
            .exec(&*sstate.database)
            .await
        else {
            return;
        };
    }

    async fn handle_packet(
        &self,
        packet: DataPacket,
        con: ClientConnectionDescriptor,
        sstate: Arc<SharedState>,
    ) -> Result<(), &'static str> {
        let prq = PlasmaRequestBundle::new(packet, con, sstate);

        // Interpret the prq.packet
        match prq.packet.packet_mode {
            PacketMode::FeslSinglePacketRequest | PacketMode::FeslMultiPacketRequest => {
                match prq.packet.mode {
                    DataMode::FESL_FSYS => {
                        match prq.packet.data.get("TXN") {
                            Some(txn) => {
                                match txn.as_str() {
                                    "Hello" => {
                                        return self.handle_rq_fsys_hello(prq).await;
                                    }
                                    "Goodbye" => {
                                        return self.handle_rq_fsys_goodbye(prq).await;
                                    }
                                    "GetPingSites" => {
                                        return self.handle_rq_fsys_getpingsites(prq).await;
                                    }
                                    _ => {
                                        println!("[FESL   ][REQ][FSYS][TXN] Unhandled TXN: {:?}, ignoring...", txn);
                                        return Ok(()); // Ignore unknown TXNs
                                    }
                                }
                            }
                            _ => {
                                println!("[FESL   ][REQ][FSYS] No TXN, ignoring...");
                                return Ok(()); // Ignore empty prq.packets TXNs
                            }
                        }
                    }
                    DataMode::FESL_ACCT => {
                        match prq.packet.data.get("TXN") {
                            Some(txn) => {
                                match txn.as_str() {
                                    "GetCountryList" => {
                                        return self.handle_rq_acct_getcountrylist(prq).await;
                                    }
                                    "NuGetTos" => {
                                        return self.handle_rq_acct_nugettos(prq).await;
                                    }
                                    "NuAddAccount" => {
                                        return self.handle_rq_acct_nuaddaccount(prq).await;
                                    }
                                    "NuLogin" => {
                                        return self.handle_rq_acct_nulogin(prq).await;
                                    }
                                    "NuAddPersona" => {
                                        return self.handle_rq_acct_nuaddpersona(prq).await;
                                    }
                                    "NuGetPersonas" => {
                                        return self.handle_rq_acct_nugetpersonas(prq).await;
                                    }
                                    "NuLoginPersona" => {
                                        return self.handle_rq_acct_nuloginpersona(prq).await;
                                    }
                                    "NuSuggestPersonas" => {
                                        return self.handle_rq_acct_nusuggestpersonas(prq).await;
                                    }
                                    "NuEntitleGame" => {
                                        return self.handle_rq_acct_nuentitlegame(prq).await;
                                    }
                                    "NuPS3Login" => {
                                        return self.handle_rq_acct_nups3login(prq).await;
                                    }
                                    "NuXBL360Login" => {
                                        return self.handle_rq_acct_nuxbl360login(prq).await;
                                    }
                                    _ => {
                                        println!("[FESL   ][REQ][ACCT][TXN] Unhandled TXN: {:?}, ignoring...", txn);
                                        return Ok(()); // Ignore unknown TXNs
                                    }
                                }
                            }
                            _ => {
                                println!("[FESL   ][REQ][ACCT] No TXN, ignoring...");
                                return Ok(()); // Ignore empty prq.packets TXNs
                            }
                        }
                    }
                    DataMode::FESL_ASSO => {
                        match prq.packet.data.get("TXN") {
                            Some(txn) => {
                                match txn.as_str() {
                                    //"GetAssociations" => {
                                    //    return self.handle_rq_asso_getassociations(prq).await;
                                    //}
                                    //"AddAssociations" => {
                                    //    return self.handle_rq_asso_addassociations(prq).await;
                                    //}
                                    _ => {
                                        println!("[FESL   ][REQ][ASSO][TXN] Unhandled TXN: {:?}, ignoring...", txn);
                                        return Ok(()); // Ignore unknown TXNs
                                    }
                                }
                            }
                            _ => {
                                println!("[FESL   ][REQ][ASSO] No TXN, ignoring...");
                                return Ok(()); // Ignore empty prq.packets TXNs
                            }
                        }
                    }
                    DataMode::FESL_PRES => {
                        match prq.packet.data.get("TXN") {
                            Some(txn) => {
                                match txn.as_str() {
                                    "SetPresenceStatus" => {
                                        return self.handle_rq_pres_setpresencestatus(prq).await;
                                    }
                                    _ => {
                                        println!("[FESL   ][REQ][PRES][TXN] Unhandled TXN: {:?}, ignoring...", txn);
                                        return Ok(()); // Ignore unknown TXNs
                                    }
                                }
                            }
                            _ => {
                                println!("[FESL   ][REQ][PRES] No TXN, ignoring...");
                                return Ok(()); // Ignore empty prq.packets TXNs
                            }
                        }
                    }
                    DataMode::FESL_RANK => {
                        match prq.packet.data.get("TXN") {
                            Some(tnx) => {
                                match tnx.as_str() {
                                    "GetTopNAndMe" => {
                                        return self.handle_rq_rank_gettopnandme(prq).await;
                                    }
                                    _ => {
                                        println!("[FESL   ][REQ][RANK][TXN] Unhandled TXN: {:?}, ignoring...", tnx);
                                        return Ok(()); // Ignore unknown TXNs
                                    }
                                }
                            }
                            _ => {
                                println!("[FESL   ][REQ][RANK] No TXN, ignoring...");
                                return Ok(()); // Ignore empty prq.packets TXNs
                            }
                        }
                    }
                    _ => {
                        println!(
                            "[FESL   ][REQ] Unhandled DataMode: {:?}, ignoring...",
                            &prq.packet.mode
                        );
                        return Ok(());
                    }
                }
            }
            PacketMode::FeslSinglePacketResponse | PacketMode::FeslMultiPacketResponse => {
                match prq.packet.mode {
                    DataMode::FESL_FSYS => {
                        match prq.packet.data.get("TXN") {
                            Some(txn) => {
                                match txn.as_str() {
                                    "MemCheck" => {
                                        return self.handle_rsp_fsys_memcheck(prq).await;
                                    }
                                    "Ping" => {
                                        return self.handle_rsp_fsys_ping(prq).await;
                                    }
                                    _ => {
                                        println!("[FESL   ][RSP][FSYS][TXN] Unhandled TXN: {:?}, ignoring...", txn);
                                        return Ok(()); // Ignore unknown TXNs
                                    }
                                }
                            }
                            _ => {
                                println!("[FESL   ][RSP][FSYS] No TXN, ignoring...");
                                return Ok(()); // Ignore empty prq.packets TXNs
                            }
                        }
                    }
                    _ => {
                        println!(
                            "[FESL   ][RSP] Unhandled DataMode: {:?}, ignoring...",
                            &prq.packet.mode
                        );
                        return Ok(());
                    }
                }
            }
            _ => {
                println!(
                    "[FESL   ]Unhandled PacketMode: {:?}, ignoring...",
                    &prq.packet.packet_mode
                );
                return Ok(());
            }
        }
    }
}

impl FeslHandler {
    async fn handle_rq_fsys_hello(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        fsys_hello(&self, prq).await
    }

    async fn handle_rq_fsys_goodbye(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        fsys_goodbye(&self, prq).await
    }

    async fn handle_rq_fsys_getpingsites(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        fsys_getpingsites(&self, prq).await
    }

    async fn handle_rq_acct_getcountrylist(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_getcountrylist(&self, prq).await
    }

    async fn handle_rq_acct_nugettos(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nugettos(&self, prq).await
    }

    async fn handle_rq_acct_nuaddaccount(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nuaddaccount(&self, prq).await
    }

    async fn handle_rq_acct_nulogin(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nulogin(&self, prq).await
    }

    async fn handle_rq_acct_nuentitlegame(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nuentitlegame(&self, prq).await
    }

    async fn handle_rq_acct_nugetpersonas(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nugetpersona(&self, prq).await
    }

    async fn handle_rq_acct_nusuggestpersonas(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nusuggestpersonas(&self, prq).await
    }

    async fn handle_rq_acct_nuloginpersona(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nuloginpersona(&self, prq).await
    }

    async fn handle_rq_acct_nuaddpersona(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nuaddpersona(&self, prq).await
    }

    async fn handle_rq_asso_getassociations(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        asso_getassociations(&self, prq).await
    }

    async fn handle_rq_asso_addassociations(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        asso_addassociations(&self, prq).await
    }

    async fn handle_rq_pres_setpresencestatus(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        pres_setpresencestatus(&self, prq).await
    }

    async fn handle_rsp_fsys_memcheck(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        fsys_memcheck(&self, prq).await
    }

    async fn handle_rsp_fsys_ping(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        fsys_ping(&self, prq).await
    }

    async fn send_memcheck(
        &self,
        con: &ClientConnectionDescriptor,
        sstate: &Arc<SharedState>,
        delay: i64,
    ) -> Result<(), &'static str> {
        send_memcheck(&self, con, sstate, delay).await
    }

    async fn send_ping(
        &self,
        con: &ClientConnectionDescriptor,
        sstate: &Arc<SharedState>,
        delay: i64,
    ) -> Result<(), &'static str> {
        send_ping(&self, con, sstate, delay).await
    }

    async fn handle_rq_acct_nups3login(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nups3login(&self, prq).await
    }

    async fn handle_rq_rank_gettopnandme(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        rank_gettopnandme(&self, prq).await
    }

    async fn handle_rq_acct_nuxbl360login(
        &self,
        mut prq: PlasmaRequestBundle,
    ) -> Result<(), &'static str> {
        acct_nuxbl360login(&self, prq).await
    }
}
