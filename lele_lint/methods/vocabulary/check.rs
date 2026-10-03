use std::path::Path;

use syn::visit::Visit;

use crate::checkers;
use crate::common;
use crate::Diagnostic;
use crate::Project;

pub fn check(_self: &checkers::vocabulary::Vocabulary, project: &Project) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if project.vocabulary.is_empty() {
        return diags;
    }
    validate_config(project, &mut diags);
    let terms = build_terms(project);
    scan_paths(project, &terms, &mut diags);
    scan_identifiers(project, &terms, &mut diags);
    diags
}

// needed helper: one banned term prepared for contiguous word matching
struct BannedTerm<'a> {
    name: &'a str,
    meaning: &'a str,
    banned: &'a str,
    words: Vec<String>,
}

// needed helper: flatten every banned term with its vocabulary entry
fn build_terms(project: &Project) -> Vec<BannedTerm<'_>> {
    project
        .vocabulary
        .iter()
        .flat_map(|entry| {
            entry.banned.iter().map(move |banned| BannedTerm {
                name: &entry.name,
                meaning: &entry.meaning,
                banned,
                words: common::split_words(banned),
            })
        })
        .collect()
}

// needed helper: validate the vocabulary declarations themselves
fn validate_config(project: &Project, diags: &mut Vec<Diagnostic>) {
    let mut seen: Vec<(String, String)> = Vec::new();
    for entry in &project.vocabulary {
        if entry.banned.is_empty() {
            diags.push(config_diag(
                project,
                &format!("vocabulary `{}` has an empty `banned` list", entry.name),
            ));
            continue;
        }
        let name_words = common::split_words(&entry.name);
        for banned in &entry.banned {
            let banned_words = common::split_words(banned);
            if contains_contiguous(&name_words, &banned_words) {
                diags.push(config_diag(
                    project,
                    &format!(
                        "vocabulary `{}` bans its own name via `{banned}`",
                        entry.name
                    ),
                ));
            }
            let key = banned_words.join("_");
            if let Some((_, other)) = seen.iter().find(|(k, n)| *k == key && *n != entry.name) {
                diags.push(config_diag(
                    project,
                    &format!(
                        "banned word `{banned}` is declared by both `{other}` and `{}`",
                        entry.name
                    ),
                ));
            } else {
                seen.push((key, entry.name.clone()));
            }
        }
    }
}

// needed helper: a config error is reported on `lele.toml` line 1
fn config_diag(project: &Project, message: &str) -> Diagnostic {
    Diagnostic {
        file: project.root.join("lele.toml"),
        line: 1,
        col: 0,
        code: "E035".to_string(),
        message: message.to_string(),
    }
}

// needed helper: scan file and folder names under `src/` and `methods/`
fn scan_paths(project: &Project, terms: &[BannedTerm], diags: &mut Vec<Diagnostic>) {
    for entry in &project.entries {
        let file = project.src_dir.join(&entry.relative_path);
        scan_path_components(&entry.relative_path, &file, terms, diags);
    }
    if let Some(methods_dir) = &project.methods_dir {
        for entry in &project.methods_entries {
            let file = methods_dir.join(&entry.relative_path);
            scan_path_components(&entry.relative_path, &file, terms, diags);
        }
    }
}

// needed helper: report banned words in every path component
fn scan_path_components(
    path: &Path,
    file: &Path,
    terms: &[BannedTerm],
    diags: &mut Vec<Diagnostic>,
) {
    for component in path.components() {
        let Some(raw) = component.as_os_str().to_str() else {
            continue;
        };
        let name = raw.strip_suffix(".rs").unwrap_or(raw);
        if let Some(term) = find_term(&common::split_words(name), terms) {
            diags.push(term_diag(file, name, 1, term));
        }
    }
}

// needed helper: scan every declared identifier in `src/` and `methods/` files
fn scan_identifiers(project: &Project, terms: &[BannedTerm], diags: &mut Vec<Diagnostic>) {
    for (rel_path, file) in &project.parsed_files {
        scan_file(&project.src_dir.join(rel_path), file, terms, diags);
    }
    if let Some(methods_dir) = &project.methods_dir {
        for (rel_path, file) in &project.methods_parsed_files {
            scan_file(&methods_dir.join(rel_path), file, terms, diags);
        }
    }
}

// needed helper: collect identifier hits in one file and report them
fn scan_file(file: &Path, parsed: &syn::File, terms: &[BannedTerm], diags: &mut Vec<Diagnostic>) {
    let mut visitor = IdentVisitor::default();
    visitor.visit_file(parsed);
    for hit in &visitor.hits {
        if let Some(term) = find_term(&common::split_words(&hit.name), terms) {
            diags.push(term_diag(file, &hit.name, hit.line, term));
        }
    }
}

// needed helper: build the diagnostic message for one hit
fn term_diag(file: &Path, ident: &str, line: usize, term: &BannedTerm) -> Diagnostic {
    let name = term.name;
    let meaning = term.meaning;
    let banned = term.banned;
    Diagnostic {
        file: file.to_path_buf(),
        line,
        col: 0,
        code: "E035".to_string(),
        message: format!(
            "`{ident}` uses banned word `{banned}` — use `{name}` ({name}: {meaning})"
        ),
    }
}

// needed helper: first banned term whose words appear contiguously in `words`
fn find_term<'a, 'b>(words: &[String], terms: &'a [BannedTerm<'b>]) -> Option<&'a BannedTerm<'b>> {
    terms
        .iter()
        .find(|term| contains_contiguous(words, &term.words))
}

// needed helper: contiguous subslice test
fn contains_contiguous(haystack: &[String], needle: &[String]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

// needed helper: identifier name plus source line
struct IdentHit {
    name: String,
    line: usize,
}

#[derive(Default)]
struct IdentVisitor {
    hits: Vec<IdentHit>,
    in_trait_impl: bool,
}

impl IdentVisitor {
    fn push(&mut self, ident: &syn::Ident) {
        self.hits.push(IdentHit {
            name: ident.to_string(),
            line: ident.span().start().line,
        });
    }
}

impl<'ast> Visit<'ast> for IdentVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.push(&node.sig.ident);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.push(&node.ident);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.push(&node.ident);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        self.push(&node.ident);
        syn::visit::visit_variant(self, node);
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(ident) = &node.ident {
            self.push(ident);
        }
        syn::visit::visit_field(self, node);
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.push(&node.ident);
        syn::visit::visit_item_const(self, node);
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        self.push(&node.ident);
        syn::visit::visit_item_static(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.push(&node.ident);
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.push(&node.ident);
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.push(&node.sig.ident);
        syn::visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.push(&node.ident);
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let was_in_trait_impl = self.in_trait_impl;
        self.in_trait_impl = node.trait_.is_some();
        syn::visit::visit_item_impl(self, node);
        self.in_trait_impl = was_in_trait_impl;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if !self.in_trait_impl {
            self.push(&node.sig.ident);
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.push(&node.ident);
        syn::visit::visit_pat_ident(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::contains_contiguous;

    #[test]
    fn test_usage() {
        let haystack = vec!["run".to_string(), "board".to_string()];
        assert!(contains_contiguous(&haystack, &["board".to_string()]));
        assert!(!contains_contiguous(&haystack, &["keyboard".to_string()]));
        let multi = vec!["run".to_string(), "lobby".to_string(), "room".to_string()];
        assert!(contains_contiguous(
            &multi,
            &["lobby".to_string(), "room".to_string()]
        ));
    }
}
