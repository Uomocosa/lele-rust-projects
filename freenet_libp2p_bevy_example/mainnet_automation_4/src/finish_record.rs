use std::fs;
use std::path::Path;
use std::process::Child;

use crate::Error;

pub fn finish_record(mut child: Child, out: &Path) -> Result<Vec<u8>, Error> {
    let status = child
        .wait()
        .map_err(|e| Error::Ffmpeg(format!("waiting for ffmpeg: {e}")))?;
    if !status.success() {
        return Err(Error::Ffmpeg("ffmpeg exited nonzero".to_string()));
    }
    fs::read(out).map_err(|e| Error::Ffmpeg(format!("reading recording {}: {e}", out.display())))
}

// no test_usage necessary
