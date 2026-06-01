use bevy::prelude::MessageWriter;

use crate::Event;
use crate::resource::FreenetNode;

pub fn poll_events(events: MessageWriter<Event>, node: Option<bevy::prelude::Res<FreenetNode>>) {
    let _ = (events, node);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::Event;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<Event>();
        app.add_systems(Update, super::poll_events);
        app.update();
    }
}
