pub mod build_game;
pub use build_game::build_game;

pub mod creds;
pub use creds::Creds;

pub mod drive_cursor;
pub use drive_cursor::drive_cursor;

pub mod drive_random;
pub use drive_random::drive_random;

pub mod finish_record;
pub use finish_record::finish_record;

pub mod load_creds;
pub use load_creds::load_creds;

pub mod bevy_systems;

pub mod poke;
pub use poke::poke;

pub mod place_window;
pub use place_window::place_window;

pub mod require_xterm;
pub use require_xterm::require_xterm;

pub mod send_photo;
pub use send_photo::send_photo;

pub mod send_photo_file;
pub use send_photo_file::send_photo_file;

pub mod send_video;
pub use send_video::send_video;

pub mod send_video_file;
pub use send_video_file::send_video_file;

pub mod spawn_xterm;
pub use spawn_xterm::spawn_xterm;

pub mod start_record;
pub use start_record::start_record;

pub mod start_record_at;
pub use start_record_at::start_record_at;

pub mod terminal_guard;
pub use terminal_guard::TerminalGuard;

mod terminal_guard_title;

pub mod tile_three;
pub use tile_three::tile_three;

pub mod wakeup_screen;
pub use wakeup_screen::wakeup_screen;

pub mod assert_count_eq;
pub use assert_count_eq::assert_count_eq;

pub mod click_times;
pub use click_times::click_times;

pub mod fixture;
pub use fixture::fixture;

pub mod get_count;
pub use get_count::get_count;

pub mod get_global;
pub use get_global::get_global;

pub mod tick_until_merged;
pub use tick_until_merged::tick_until_merged;

pub mod ui_test_plugin;
pub use ui_test_plugin::UiTestPlugin;
