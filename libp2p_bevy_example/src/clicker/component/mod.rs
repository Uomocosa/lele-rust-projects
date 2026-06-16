pub mod ClickCounterMethod;
pub mod OwnerMethod;
#[path = "ClickCounter.rs"]
pub mod click_counter;
#[path = "ClickTarget.rs"]
pub mod click_target;
#[path = "Owner.rs"]
pub mod owner;

pub use click_counter::ClickCounter;
pub use click_target::ClickTarget;
pub use owner::Owner;
