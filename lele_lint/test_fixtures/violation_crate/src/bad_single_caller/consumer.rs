use super::msg::Msg;

pub fn handle(msg: Msg) {
    match msg {
        Msg::Ping => {}
        Msg::Pong => {}
    }
}

#[cfg(test)]
mod tests {
    use super::handle;
    use super::msg::Msg;

    #[test]
    fn test_usage() {
        handle(Msg::Ping);
    }
}