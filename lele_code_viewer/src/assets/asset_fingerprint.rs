use crate::assets;

pub fn asset_fingerprint(name: &str) -> String {
    let bytes: &[u8] = match name {
        "style.css" => assets::style_css().as_bytes(),
        _ => assets::app_js().as_bytes(),
    };
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:x}")
}

#[cfg(test)]
mod tests {
    use super::asset_fingerprint;

    #[test]
    fn test_usage() {
        let css = asset_fingerprint("style.css");
        let js = asset_fingerprint("app.js");
        assert_ne!(css, "");
        assert_ne!(js, "");
        assert_ne!(css, js);
        assert_eq!(css, asset_fingerprint("style.css"));
    }
}
