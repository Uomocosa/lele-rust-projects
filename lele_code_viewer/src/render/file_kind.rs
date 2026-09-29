use std::path::Path;

use crate::render;

pub fn file_kind(path: &Path) -> render::FileKind {
    if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        let ext = ext.to_ascii_lowercase();
        if ext == "rs" {
            return render::FileKind::Rust;
        }
        if is_text_ext(&ext) {
            return render::FileKind::Text;
        }
        if is_image_ext(&ext) {
            return render::FileKind::Image;
        }
        if is_video_ext(&ext) {
            return render::FileKind::Video;
        }
        if is_audio_ext(&ext) {
            return render::FileKind::Audio;
        }
        return render::FileKind::Other;
    }
    if is_text_name(path) {
        return render::FileKind::Text;
    }
    render::FileKind::Other
}

// needed helper: extensions rendered as highlighted text
fn is_text_ext(ext: &str) -> bool {
    matches!(
        ext,
        "toml"
            | "json"
            | "yaml"
            | "yml"
            | "txt"
            | "log"
            | "csv"
            | "tsv"
            | "ini"
            | "cfg"
            | "conf"
            | "css"
            | "js"
            | "mjs"
            | "cjs"
            | "ts"
            | "tsx"
            | "html"
            | "htm"
            | "xml"
            | "md"
            | "markdown"
            | "lock"
            | "sh"
            | "bash"
            | "zsh"
            | "py"
            | "c"
            | "h"
            | "cpp"
            | "hpp"
            | "java"
            | "go"
            | "rb"
            | "nix"
            | "ron"
    )
}

// needed helper: extensions embedded as images
fn is_image_ext(ext: &str) -> bool {
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" | "bmp" | "avif"
    )
}

// needed helper: extensions embedded as video
fn is_video_ext(ext: &str) -> bool {
    matches!(ext, "mp4" | "webm" | "ogv" | "mov")
}

// needed helper: extensions embedded as audio
fn is_audio_ext(ext: &str) -> bool {
    matches!(ext, "mp3" | "wav" | "ogg" | "oga" | "flac" | "m4a")
}

// needed helper: extensionless names treated as text
fn is_text_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                "Dockerfile"
                    | "Makefile"
                    | "Gemfile"
                    | "Rakefile"
                    | ".gitignore"
                    | ".dockerignore"
                    | ".env"
                    | ".editorconfig"
                    | "LICENSE"
                    | "README"
                    | "CHANGELOG"
            )
        })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::file_kind;
    use crate::render;

    #[test]
    fn test_usage() {
        assert_eq!(file_kind(Path::new("src/a.rs")), render::FileKind::Rust);
        assert_eq!(file_kind(Path::new("Cargo.toml")), render::FileKind::Text);
        assert_eq!(file_kind(Path::new("shot.PNG")), render::FileKind::Image);
        assert_eq!(file_kind(Path::new("clip.mp4")), render::FileKind::Video);
        assert_eq!(file_kind(Path::new("song.ogg")), render::FileKind::Audio);
        assert_eq!(file_kind(Path::new("Dockerfile")), render::FileKind::Text);
        assert_eq!(file_kind(Path::new("a.bin")), render::FileKind::Other);
    }
}
