use super::super::Mesh;
use super::super::Player;
use super::step::Step;

/// # Errors
/// Returns an error when the mesh cannot reach the scenario's expected totals.
pub fn run_on_mesh(mesh: &mut Mesh, steps: &[Step]) -> Result<(), String> {
    for step in steps {
        match step {
            Step::Click { player, times } => mesh.peer(Player(*player)).clicks(*times),
            Step::Partition { first, second } => {
                let a = index_of(mesh, *first)?;
                let b = index_of(mesh, *second)?;
                mesh.partition(a, b);
            }
            Step::Heal { first, second } => {
                let a = index_of(mesh, *first)?;
                let b = index_of(mesh, *second)?;
                mesh.heal(a, b);
            }
            Step::ExpectGlobal { total } => {
                for _ in 0..Mesh::CONVERGE_TICKS {
                    mesh.step();
                }
                for count in mesh.counts() {
                    if count.global != *total {
                        return Err(format!("global {} != expected {total}", count.global));
                    }
                }
            }
        }
    }
    Ok(())
}

// needed helper: maps a player number onto its mesh index
fn index_of(mesh: &Mesh, player: u64) -> Result<usize, String> {
    mesh.index_of(Player(player))
        .ok_or_else(|| format!("no mesh peer for player {player}"))
}

#[cfg(test)]
mod tests {
    use super::run_on_mesh;
    use crate::testing::Mesh;
    use crate::testing::scenario::rejoin_scenario;

    #[test]
    fn test_usage() {
        let mut mesh = Mesh::of(3);
        let steps = rejoin_scenario(15);
        assert!(run_on_mesh(&mut mesh, &steps).is_ok());
    }
}
