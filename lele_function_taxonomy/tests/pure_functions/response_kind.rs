pub const fn response_kind_tag(value: u8) -> &'static str {
    match value {
        0 => "subscribed",
        1 => "state",
        2 => "notification",
        3 => "update",
        _ => "other",
    }
}
