use crate::WindowInfo;

/// ImageMagick-style geometry string, e.g. "1200x800+720+240".
pub fn geometry(window: &WindowInfo) -> String {
    format!(
        "{}x{}+{}+{}",
        window.width,
        window.height,
        window.x.max(0),
        window.y.max(0)
    )
}

#[cfg(test)]
mod tests {
    use super::geometry;
    use crate::WindowInfo;

    #[test]
    fn test_usage() {
        let window = WindowInfo {
            id: "0x1".to_string(),
            desktop: 0,
            pid: 1,
            x: -10,
            y: 5,
            width: 100,
            height: 50,
            host: "h".to_string(),
            title: "t".to_string(),
        };
        assert_eq!(geometry(&window), "100x50+0+5");
    }
}
