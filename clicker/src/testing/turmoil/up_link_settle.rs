use futures_util::future::LocalBoxFuture;

use super::constants::STEP_SLEEP;
use super::up_link::UpLink;

pub fn settle<'a>(link: &'a mut UpLink, name: &'a str, iters: u32) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        for _ in 0..iters {
            link.pump(name).await;
            tokio::time::sleep(STEP_SLEEP).await;
        }
    })
}

// no test_usage necessary
