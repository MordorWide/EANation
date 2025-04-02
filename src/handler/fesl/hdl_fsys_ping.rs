use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;
use crate::handler::fesl::FESL_PING_INTERVAL;

pub async fn fsys_ping(fh: &FeslHandler, mut prq: PlasmaRequestBundle) -> Result<(), &'static str> {
    // We just received a ping request
    let _ = fh
        .send_ping(&prq.con, &prq.sstate, FESL_PING_INTERVAL as i64)
        .await;
    return Ok(());
}
