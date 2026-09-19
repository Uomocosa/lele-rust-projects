use futures_util::future::LocalBoxFuture;

use super::constants::STEP_SLEEP;
use super::done::Done;
use super::up_link::UpLink;

pub fn park_until_done<'a>(
    link: &'a mut UpLink,
    name: &'a str,
    done: &'a Done,
) -> LocalBoxFuture<'a, ()> {
    Box::pin(async move {
        loop {
            if *done.borrow() {
                break;
            }
            link.pump(name).await;
            tokio::time::sleep(STEP_SLEEP).await;
        }
    })
}

// no test_usage necessary
