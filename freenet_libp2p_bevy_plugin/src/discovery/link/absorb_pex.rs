use super::absorb_pex_resp::absorb_pex_resp;
use super::pex_msg::PexMsg;
use super::pex_response::pex_response;
use super::publish_pex::publish_pex;
use super::run_context::RunContext;

pub fn absorb_pex(ctx: &mut RunContext, from: &str, data: &[u8]) {
    let Ok(msg) = bincode::deserialize::<PexMsg>(data) else {
        return;
    };
    match msg {
        PexMsg::Ask => publish_pex(&ctx.net_tx, &ctx.id, &pex_response(ctx)),
        PexMsg::Resp { peers, rooms } => absorb_pex_resp(ctx, from, &peers, &rooms),
    }
}

// no test_usage necessary
