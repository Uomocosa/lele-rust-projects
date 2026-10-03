pub trait Probe {
    fn read(&self) -> Option<String>;
}

pub struct EnvProbe;

impl Probe for EnvProbe {
    fn read(&self) -> Option<String> {
        std::env::var("FIXTURE_ENV").ok()
    }
}

pub fn probe_env() -> Option<String> {
    EnvProbe.read()
}
