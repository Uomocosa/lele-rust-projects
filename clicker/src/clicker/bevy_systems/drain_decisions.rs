use std::io::Write;

use bevy::prelude::*;

use crate::clicker;

pub fn drain_decisions(mut log: ResMut<clicker::DecisionLog>) {
    let lines = clicker::DecisionLog::take();
    let Some(file) = (*log).as_mut() else {
        return;
    };
    for line in lines {
        let _ = writeln!(file, "{line}");
    }
    let _ = file.flush();
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek};

    use bevy::prelude::*;

    use super::drain_decisions;
    use crate::clicker;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let file = tempfile::tempfile().expect("decision drain test tempfile");
        let mut probe = file.try_clone().expect("decision file probe");
        app.insert_resource(clicker::DecisionLog(Some(file)));
        clicker::DecisionLog::record("sync: req sent on connect");
        app.add_systems(Update, drain_decisions);
        app.update();
        probe
            .seek(std::io::SeekFrom::Start(0))
            .expect("decision file rewind");
        let mut content = String::new();
        probe
            .read_to_string(&mut content)
            .expect("decision file readable");
        assert!(
            content.contains("sync: req sent on connect"),
            "drained decisions land in the file, got {content:?}"
        );
    }
}
