mod inventory;
pub mod bevy_systems;

pub use bevy_systems::poll_inv; // RE-EXPORTED at domain root
pub use inventory::Inventory;
