use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::theater::TheaterHandler;
use crate::handler::theater::THEATER_PING_INTERVAL;


pub async fn handle_rsp_ping(
    fh: &TheaterHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // Re-enqueue the next ping request
    let _ = fh
        .send_ping(&prq.con, &prq.sstate, THEATER_PING_INTERVAL as i64)
        .await;
    Ok(())
}
