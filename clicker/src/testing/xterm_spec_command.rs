use super::shell_escape::shell_escape;
use super::xterm_spec::XtermSpec;

#[must_use]
pub fn command(spec: &XtermSpec) -> String {
    let bin_str = spec.bin.to_string_lossy().to_string();
    let log_str = spec.log.to_string_lossy().to_string();
    let create_arg = if spec.create { " --create-lobby" } else { "" };
    let lobby_arg = spec.lobby.map_or(String::new(), |room| {
        format!(" --lobby {}", shell_escape(room))
    });
    let since_arg = spec
        .since_epoch
        .map_or(String::new(), |epoch| format!(" --since-epoch {epoch}"));
    let mdns_arg = if spec.mdns {
        String::new()
    } else {
        " --disable-mdns".to_string()
    };
    format!(
        "stdbuf -oL -eL {} --namespace {} {}{} --instance-tag {} --own-id {} --contract-params {}{} --transport {}{} 2>&1 | tee -a {}; echo \"[clicker-3 #{} exited $?]\"; exec bash",
        shell_escape(&bin_str),
        shell_escape(spec.namespace),
        lobby_arg,
        create_arg,
        spec.tag,
        spec.tag,
        shell_escape(spec.contract_params),
        since_arg,
        shell_escape(spec.transport),
        mdns_arg,
        shell_escape(&log_str),
        spec.tag
    )
}

// no test_usage necessary
