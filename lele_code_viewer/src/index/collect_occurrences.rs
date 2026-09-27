use std::path::Path;

use derive_more::Deref;
use derive_more::DerefMut;
use syn::spanned::Spanned;
use syn::visit::Visit;

use crate::index;

struct RawOcc {
    line: usize,
    start: usize,
    end: usize,
    segments: Vec<String>,
}

#[derive(Deref, DerefMut)]
struct Collector<'a>(&'a mut Vec<RawOcc>);

impl<'ast> syn::visit::Visit<'ast> for Collector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let start = path.span().start();
        let end = path.span().end();
        if start.line == end.line && !path.segments.is_empty() {
            let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
            self.push(RawOcc {
                line: start.line,
                start: start.column,
                end: end.column,
                segments,
            });
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let start = call.method.span().start();
        let end = call.method.span().end();
        self.push(RawOcc {
            line: start.line,
            start: start.column,
            end: end.column,
            segments: vec![call.method.to_string()],
        });
        syn::visit::visit_expr_method_call(self, call);
    }
}

pub fn collect_occurrences(
    rel: &Path,
    file: &syn::File,
    imports: &index::Imports,
    idx: &index::SymbolIndex,
) -> Vec<index::Occurrence> {
    let mut raw_occurrences: Vec<RawOcc> = Vec::new();
    {
        let mut collector = Collector(&mut raw_occurrences);
        collector.visit_file(file);
    }
    let mut out: Vec<index::Occurrence> = Vec::new();
    for raw in &raw_occurrences {
        out.push(index::Occurrence {
            line: raw.line,
            start_col: raw.start,
            end_col: raw.end,
            target: index::resolve(&raw.segments, imports, idx),
            is_self: false,
        });
    }
    for item in &idx.items {
        if item.file == rel {
            out.push(index::Occurrence {
                line: item.name_line,
                start_col: item.name_col_start,
                end_col: item.name_col_end,
                target: Some(item.id.clone()),
                is_self: true,
            });
        }
    }
    out.sort_by_key(|o| (o.line, o.start_col));
    out.dedup_by_key(|o| (o.line, o.start_col));
    out
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::collect_occurrences;
    use crate::index;

    #[test]
    fn test_usage() {
        let text = "pub fn f() { let x = foo::bar(); baz(1); }\n";
        let file: syn::File = syn::parse_str(text).unwrap();
        let idx = index::SymbolIndex::default();
        let imports = index::Imports::default();
        let occs = collect_occurrences(Path::new("src/a.rs"), &file, &imports, &idx);
        assert!(occs.len() >= 2);
    }
}
