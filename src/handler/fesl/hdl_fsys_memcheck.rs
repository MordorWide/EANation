use crate::plasma_handle::PlasmaRequestBundle;
use crate::handler::fesl::FeslHandler;


const FESL_MEMCHECK_INTERVAL: u32 = 120; // 2 mins

pub async fn fsys_memcheck(
    fh: &FeslHandler,
    mut prq: PlasmaRequestBundle,
) -> Result<(), &'static str> {
    // We just received a memcheck request

    let _ = fh
        .send_memcheck(&prq.con, &prq.sstate, FESL_MEMCHECK_INTERVAL as i64)
        .await;
    return Ok(());
}
