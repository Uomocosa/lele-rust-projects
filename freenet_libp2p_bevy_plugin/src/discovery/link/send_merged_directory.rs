use super::run_context::RunContext;

pub fn send_merged_directory(ctx: &RunContext) {
    let mut view = ctx.directory.slots.clone();
    for (name, entry) in &ctx.known_rooms {
        let keep = view
            .get(name)
            .is_none_or(|known| entry.updated_at >= known.updated_at);
        if keep {
            view.insert(name.clone(), entry.clone());
        }
    }
    ctx.directory_tx.send(view).ok();
}

// no test_usage necessary
