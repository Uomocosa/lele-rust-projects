use futures_util::future::LocalBoxFuture;

use super::constants::{HANDSHAKE_ITERS, STEP_SLEEP};
use super::labeled::labeled;
use super::up_link::UpLink;

pub fn handshake<'a>(link: &'a mut UpLink, name: &'a str) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        for _ in 0..HANDSHAKE_ITERS {
            link.pump(name).await;
            if labeled(&mut link.app) >= 3 {
                break;
            }
            tokio::time::sleep(STEP_SLEEP).await;
        }
    })
}

// no test_usage necessary
