use std::path::Path;

use super::xterm_spec_command;

pub struct XtermSpec<'a> {
    pub bin: &'a Path,
    pub namespace: &'a str,
    pub lobby: Option<&'a str>,
    pub create: bool,
    pub tag: u64,
    pub contract_params: &'a str,
    pub since_epoch: Option<u64>,
    pub transport: &'a str,
    pub mdns: bool,
    pub brp_port: Option<u16>,
    pub log: &'a Path,
}

#[rustfmt::skip]
impl XtermSpec<'_> {
    #[must_use]
    pub fn command(&self) -> String { xterm_spec_command::command(self) }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::XtermSpec;

    #[test]
    fn test_usage() {
        let spec = XtermSpec {
            bin: Path::new("/bin/true"),
            namespace: "ns",
            lobby: None,
            create: false,
            tag: 1,
            contract_params: "params",
            since_epoch: None,
            transport: "tcp",
            mdns: true,
            brp_port: None,
            log: Path::new("/tmp/x.log"),
        };
        assert!(spec.command().contains("--namespace"));
    }
}
