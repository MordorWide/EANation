use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;


const THEATER_PING_INTERVAL: u32 = 60; // 1 min

pub async fn handle_rsp_ping(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    let _ = fh
        .send_ping(&prq.con, &prq.sstate, THEATER_PING_INTERVAL as i64)
        .await;
    Ok(())
}
