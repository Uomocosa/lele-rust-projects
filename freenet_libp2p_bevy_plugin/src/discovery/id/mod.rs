pub use crate::discovery::basic::newtypes::{
    EpochSecs, GameName, GameToken, RemotePeerId, RoomName,
};
pub use crate::discovery::basic::structs::{Presence, RoomRecord};

pub mod now_epoch;
pub use now_epoch::now_epoch;

pub mod occupancy;
pub use occupancy::occupancy;

pub mod unique_game_id;
pub use unique_game_id::UniqueGameId;
