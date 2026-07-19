use bevy::app::AppExit;
use bevy::prelude::*;

use crate::clicker::headless::State::HeadlessState;
use crate::clicker::message::CountChanged::CountChanged;
use crate::clicker::resource::State::ClickerState;

pub fn headless_counter(
    state: Res<ClickerState>,
    mut count_reader: MessageReader<CountChanged>,
    mut hs: ResMut<HeadlessState>,
    mut app_exit: MessageWriter<AppExit>,
) {
    for _ in count_reader.read() {
        hs.pending = false;
        hs.completed += 1;
        println!("tick {}, count={}", hs.completed, state.count);
        if hs.completed >= hs.max_ticks {
            app_exit.write(AppExit::Success);
        }
    }
}
