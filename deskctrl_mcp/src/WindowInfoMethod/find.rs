use crate::{Error, WindowInfo, WindowInfoMethod::is_valid_id};

/// Resolve a window from the selectors, first non-`None` wins: id, then pid, then title.
///
/// Returns `Ok(None)` when no selector was given (the caller then captures the full screen).
/// An ambiguous pid/title is an error listing the candidates rather than a silent pick.
pub fn find(
    windows: &[WindowInfo],
    window_id: Option<&str>,
    pid: Option<u32>,
    title: Option<&str>,
) -> Result<Option<WindowInfo>, Error> {
    if let Some(id) = window_id {
        if !is_valid_id(id) {
            return Err(Error::Window(format!(
                "invalid window_id {id:?}: expected hex like \"0x03a00004\" (see list_windows)"
            )));
        }
        let id = id.to_lowercase();
        // wmctrl zero-pads ids; compare numerically so "0x3a00004" also matches.
        let wanted = u64::from_str_radix(id.trim_start_matches("0x"), 16).ok();
        let found = windows
            .iter()
            .find(|w| u64::from_str_radix(w.id.trim_start_matches("0x"), 16).ok() == wanted);
        // Not every window is listed by wmctrl (override-redirect, other desktops); still try
        // to capture it directly, with unknown geometry.
        return Ok(Some(found.cloned().unwrap_or(WindowInfo {
            id,
            desktop: 0,
            pid: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            host: String::new(),
            title: "<not listed by wmctrl>".to_string(),
        })));
    }

    if let Some(pid) = pid {
        return pick(
            windows.iter().filter(|w| w.pid == pid).collect(),
            &format!("pid {pid}"),
        );
    }

    if let Some(title) = title {
        let needle = title.to_lowercase();
        return pick(
            windows
                .iter()
                .filter(|w| w.title.to_lowercase().contains(&needle))
                .collect(),
            &format!("title containing {title:?}"),
        );
    }

    Ok(None)
}

fn pick(matches: Vec<&WindowInfo>, what: &str) -> Result<Option<WindowInfo>, Error> {
    match matches.len() {
        0 => Err(Error::Window(format!(
            "no window matches {what} (run list_windows to see the open windows)"
        ))),
        1 => Ok(Some(matches[0].clone())),
        _ => Err(Error::Window(format!(
            "{} windows match {what}; pass one of these as window_id:\n{}",
            matches.len(),
            matches
                .iter()
                .map(|w| format!("  {} pid={} {}", w.id, w.pid, w.title))
                .collect::<Vec<_>>()
                .join("\n")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::find;
    use crate::{Error, WindowInfo, WindowInfoMethod::parse_output};

    fn windows() -> Vec<WindowInfo> {
        parse_output(
            "0x03a00004  0 2215 720 240 1200 800 host Claude\n\
             0x04200007  0 3312 0   0   1920 1040 host Firefox — Docs\n\
             0x04200008  0 3312 10  10  800  600  host Firefox — Mail\n",
        )
    }

    #[test]
    fn test_usage() {
        let ws = windows();
        assert!(find(&ws, None, None, None).unwrap().is_none());

        let by_id = find(&ws, Some("0x3a00004"), None, None).unwrap().unwrap();
        assert_eq!(by_id.title, "Claude");

        let by_pid = find(&ws, None, Some(2215), None).unwrap().unwrap();
        assert_eq!(by_pid.id, "0x03a00004");

        let by_title = find(&ws, None, None, Some("mail")).unwrap().unwrap();
        assert_eq!(by_title.id, "0x04200008");

        let unlisted = find(&ws, Some("0xdead"), None, None).unwrap().unwrap();
        assert_eq!(unlisted.width, 0);

        assert!(matches!(
            find(&ws, Some("nope"), None, None),
            Err(Error::Window(_))
        ));
        let ambiguous = find(&ws, None, Some(3312), None).unwrap_err().to_string();
        assert!(ambiguous.contains("0x04200007") && ambiguous.contains("0x04200008"));
        assert!(matches!(
            find(&ws, None, None, Some("nothing")),
            Err(Error::Window(_))
        ));
    }
}
