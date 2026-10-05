use std::path::PathBuf;

use lele_bevy_preview::deliver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: send_one <path-to-png-or-mp4>")?;
    let caption = path.display().to_string();
    let sent = deliver::send_one::send_one(&path, &caption)?;
    if sent {
        println!("sent {}", path.display());
    } else {
        println!("no credentials; left {} on disk", path.display());
    }
    Ok(())
}
