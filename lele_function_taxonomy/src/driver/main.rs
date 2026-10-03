#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_driver::{Callbacks, Compilation};
use rustc_interface::interface;
use rustc_middle::ty::TyCtxt;

mod build_graph;
mod roots;

use build_graph::{HonestBoundary, ReportConfig};

struct TaxonomyCallbacks {
    report: bool,
}

impl Callbacks for TaxonomyCallbacks {
    fn after_analysis(&mut self, _compiler: &interface::Compiler, tcx: TyCtxt<'_>) -> Compilation {
        if self.report {
            build_graph::run(tcx);
        }
        Compilation::Continue
    }
}

fn read_config() -> ReportConfig {
    let path = std::env::var("LELE_TAXONOMY_CONFIG").unwrap_or_default();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return ReportConfig {
            honest_boundaries: Vec::new(),
            declared_dishonest: Vec::new(),
            declared_honest: Vec::new(),
        };
    };
    let parsed: ConfigToml = toml::from_str(&content).unwrap_or_default();
    let honest_boundaries = parsed
        .lele
        .boundary
        .iter()
        .filter(|b| b.require.as_deref() == Some("honest"))
        .map(|b| HonestBoundary {
            name: b.name.clone(),
            folders: b.folders.clone(),
        })
        .collect();
    ReportConfig {
        honest_boundaries,
        declared_dishonest: parsed.honesty.declared_dishonest,
        declared_honest: parsed.honesty.declared_honest,
    }
}

#[derive(serde::Deserialize, Default)]
struct ConfigToml {
    #[serde(default)]
    lele: LeleSection,
    #[serde(default)]
    honesty: Honesty,
}

#[derive(serde::Deserialize, Default)]
struct LeleSection {
    #[serde(default)]
    boundary: Vec<Boundary>,
}

#[derive(serde::Deserialize)]
struct Boundary {
    #[serde(default)]
    name: String,
    #[serde(default)]
    folders: Vec<String>,
    #[serde(default)]
    require: Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct Honesty {
    #[serde(default)]
    declared_honest: Vec<String>,
    #[serde(default)]
    declared_dishonest: Vec<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let primary = std::env::var_os("CARGO_PRIMARY_PACKAGE").is_some();
    let report = primary && std::env::var_os("LELE_TAXONOMY_CONFIG").is_some();

    let Some(real_rustc) = args.get(1) else {
        std::process::exit(1);
    };
    let rustc_args = args.get(2..).unwrap_or_default();
    let runner_args = args.get(1..).unwrap_or_default();

    if !primary {
        let code = std::process::Command::new(real_rustc)
            .args(rustc_args)
            .status()
            .ok()
            .and_then(|s| s.code())
            .unwrap_or(1);
        std::process::exit(code);
    }

    let mut callbacks = TaxonomyCallbacks { report };
    let code = rustc_driver::catch_with_exit_code(|| {
        rustc_driver::run_compiler(runner_args, &mut callbacks);
    });
    if code == std::process::ExitCode::SUCCESS {
        std::process::exit(0);
    }
    std::process::exit(1);
}
