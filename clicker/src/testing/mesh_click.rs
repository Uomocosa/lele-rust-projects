use super::mesh::Mesh;
use crate::clicker;
use crate::testing;

pub fn click(mesh: &mut Mesh, owner: u64, times: u32) {
    for app in &mut mesh.apps {
        let own = *app.world().resource::<clicker::InstanceInfo>().own_id;
        if own == owner {
            testing::click_times(app, times);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::click;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::three();
        click(&mut mesh, 2, 4);
        let counts = mesh.counts();
        assert!(counts.iter().any(|c| c.p2 == 4));
        assert!(counts.iter().all(|c| c.p1 == 0 && c.p3 == 0));
    }
}
