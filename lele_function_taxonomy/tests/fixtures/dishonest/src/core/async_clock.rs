use std::time::SystemTime;

pub async fn async_now() -> SystemTime {
    SystemTime::now()
}
