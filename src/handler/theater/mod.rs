use std::sync::Arc;
use tracing::info;

use crate::client_connection::{ClientConnectionDescriptor, ServiceType};
use crate::handler::Handler;
use crate::packet::{DataMode, DataPacket, PacketMode};
use crate::plasma_handle::PlasmaRequestBundle;
use crate::sharedstate::SharedState;

const THEATER_PING_INTERVAL: u64 = 60; // 1 min

mod hdl_conn;
use hdl_conn::handle_rq_conn;

mod hdl_user;
use hdl_user::handle_rq_user;

mod hdl_llst;
use hdl_llst::handle_rq_llst;

mod hdl_cgam;
use hdl_cgam::handle_rq_cgam;

mod hdl_ecnl;
use hdl_ecnl::handle_rq_ecnl;

mod hdl_egam;
use hdl_egam::handle_rq_egam;

mod hdl_egrs;
use hdl_egrs::handle_rq_egrs;

mod hdl_pent;
use hdl_pent::handle_rq_pent;

mod hdl_rgam;
use hdl_rgam::handle_rq_rgam;

mod hdl_plvt;
use hdl_plvt::handle_rq_plvt;

mod hdl_glst;
use hdl_glst::handle_rq_glst;

mod hdl_ubra;
use hdl_ubra::handle_rq_ubra;

mod hdl_ugam;
use hdl_ugam::handle_rsp_ugam;

mod hdl_ping;
use hdl_ping::handle_rsp_ping;

mod hdl_echo;
use hdl_echo::handle_rsp_echo;

mod utils_ping;
use utils_ping::send_ping;

pub struct TheaterHandler;

#[async_trait::async_trait]
impl Handler for TheaterHandler {
    fn handler_type(&self) -> ServiceType {
        ServiceType::Theater
    }

    async fn connection_closed(&self, con: ClientConnectionDescriptor, sstate: Arc<SharedState>) {
        // The cleanup is done in the connection_closed method of the FESL handler
    }

    async fn handle_packet(
        &self,
        packet: DataPacket,
        con: ClientConnectionDescriptor,
        sstate: Arc<SharedState>,
    ) -> Result<(), &'static str> {
        let prq = PlasmaRequestBundle::new(packet, con, sstate);

        match prq.packet.packet_mode {
            PacketMode::TheaterRequest => match prq.packet.mode {
                DataMode::THEATER_CONN => {
                    return self.handle_rq_conn(prq).await;
                }
                DataMode::THEATER_USER => {
                    return self.handle_rq_user(prq).await;
                }
                DataMode::THEATER_ECNL => {
                    return self.handle_rq_ecnl(prq).await;
                }
                DataMode::THEATER_LLST => {
                    return self.handle_rq_llst(prq).await;
                }
                DataMode::THEATER_CGAM => {
                    return self.handle_rq_cgam(prq).await;
                }
                DataMode::THEATER_EGAM => {
                    return self.handle_rq_egam(prq).await;
                }
                DataMode::THEATER_EGRS => {
                    return self.handle_rq_egrs(prq).await;
                }
                DataMode::THEATER_PENT => {
                    return self.handle_rq_pent(prq).await;
                }
                DataMode::THEATER_RGAM => {
                    return self.handle_rq_rgam(prq).await;
                }
                DataMode::THEATER_GLST => {
                    return self.handle_rq_glst(prq).await;
                }
                DataMode::THEATER_UBRA => {
                    return self.handle_rq_ubra(prq).await;
                }
                DataMode::THEATER_PLVT => {
                    return self.handle_rq_plvt(prq).await;
                }
                _ => {
                    info!(target: "theater", "Unhandled DataMode: {:?}, ignoring...", &prq.packet.mode);
                    return Ok(());
                }
            },
            PacketMode::FeslPingOrTheaterResponse => match prq.packet.mode {
                DataMode::THEATER_PING => {
                    return self.handle_rsp_ping(prq).await;
                }
                DataMode::THEATER_UGAM => {
                    return self.handle_rsp_ugam(prq).await;
                }
                DataMode::THEATER_ECHO => {
                    return self.handle_rsp_echo(prq).await;
                }
                _ => {
                    info!(target: "theater", "Unhandled DataMode: {:?}, ignoring...", &prq.packet.mode);
                    return Ok(());
                }
            },
            _ => {
                info!(target: "theater", "Unhandled PacketMode: {:?}, ignoring...", &prq.packet.packet_mode);
                return Ok(());
            }
        }
    }
}

impl TheaterHandler {
    async fn handle_rq_conn(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_conn(self, prq).await
    }

    async fn handle_rq_user(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_user(self, prq).await
    }

    async fn handle_rq_llst(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_llst(self, prq).await
    }

    async fn handle_rq_cgam(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_cgam(self, prq).await
    }

    async fn handle_rq_ecnl(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_ecnl(self, prq).await
    }

    async fn handle_rq_egam(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_egam(self, prq).await
    }

    async fn handle_rq_egrs(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_egrs(self, prq).await
    }

    async fn handle_rq_pent(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_pent(self, prq).await
    }

    async fn handle_rq_rgam(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_rgam(self, prq).await
    }

    async fn handle_rq_plvt(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_plvt(self, prq).await
    }

    async fn handle_rq_glst(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_glst(self, prq).await
    }

    async fn handle_rq_ubra(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rq_ubra(self, prq).await
    }

    async fn handle_rsp_ugam(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rsp_ugam(self, prq).await
    }

    async fn handle_rsp_ping(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rsp_ping(self, prq).await
    }

    async fn handle_rsp_echo(&self, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
        handle_rsp_echo(self, prq).await
    }

    async fn send_ping(
        &self,
        con: &ClientConnectionDescriptor,
        sstate: &Arc<SharedState>,
        delay: i64,
    ) -> Result<(), &'static str> {
        send_ping(self, con, sstate, delay).await
    }
}
