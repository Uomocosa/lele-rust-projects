use futures_util::future::LocalBoxFuture;

use super::constants::{CONVERGE_ITERS, STEP_SLEEP};
use super::counts_of::counts_of;
use super::up_link::UpLink;
use crate::testing;

pub fn converge_to<'a>(
    link: &'a mut UpLink,
    name: &'a str,
    want: testing::MeshCount,
) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        for _ in 0..CONVERGE_ITERS {
            link.pump(name).await;
            if counts_of(&mut link.app) == want {
                break;
            }
            tokio::time::sleep(STEP_SLEEP).await;
        }
    })
}

// no test_usage necessary
