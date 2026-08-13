use crate::Server;

pub fn new() -> Server {
    let gh_token = std::env::var("GH_TOKEN").ok();
    let gh_repo =
        std::env::var("GH_REPO").unwrap_or_else(|_| "Uomocosa/lele-rust-projects".to_string());
    let game_exe = std::env::var("FBX_GAME_EXE").ok();
    Server {
        gh_repo,
        gh_token,
        game_exe,
    }
}

#[cfg(test)]
mod tests {
    use super::new;

    #[test]
    fn test_usage() {
        let server = new();
        assert_eq!(server.gh_repo, "Uomocosa/lele-rust-projects");
        assert!(server.game_exe.is_none());
    }
}
