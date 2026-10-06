use crate::deliver::basic::structs::Capture;

pub struct Report {
    pub captures: Vec<Capture>,
    pub clip: Option<Capture>,
}

impl std::fmt::Display for Report {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self
            .captures
            .iter()
            .map(|capture| capture.artifact.label.as_str())
            .collect();
        write!(
            formatter,
            "{} capture(s): {}",
            self.captures.len(),
            names.join(", ")
        )
    }
}

#[rustfmt::skip]
impl Report {
    #[must_use]
    pub const fn counts(&self) -> usize { self.captures.len() }
    #[must_use]
    pub fn first(&self) -> Option<&Capture> { self.captures.first() }
}

#[cfg(test)]
mod tests {
    use crate::deliver::basic::enums::{Media, Status};
    use crate::deliver::basic::structs::{Artifact, Capture};
    use crate::report::Report;
    use std::path::PathBuf;

    // needed helper: fixture shared by the tests in this file
    fn capture(label: &str, status: Status) -> Capture {
        Capture {
            path: PathBuf::from(format!("out/{label}.png")),
            artifact: Artifact {
                scene: String::from("ui"),
                label: label.to_string(),
                fingerprint: format!("ui|{label}"),
                pixel_hash: String::from("h"),
                media: Media::Png,
                kind: crate::scene::Kind::System,
            },
            status,
        }
    }

    #[test]
    fn test_usage() {
        let report = Report {
            captures: vec![
                capture("rest", Status::Same),
                capture("hover", Status::Changed),
            ],
            clip: Some(capture("clip", Status::Changed)),
        };
        let text = report.to_string();
        assert!(text.contains("2 capture(s)"));
        assert!(text.contains("rest"));
        assert!(text.contains("hover"));
        assert_eq!(report.counts(), 2);
        assert!(report.first().is_some());
        assert!(report.clip.is_some());
    }
}
