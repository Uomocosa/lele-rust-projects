pub fn remove_if<F>(items: &mut Vec<u32>, mut pred: F) -> usize
where
    F: FnMut(&u32) -> bool,
{
    let mut boundary = 0;
    for i in 0..items.len() {
        if pred(&items[i]) {
            items.swap(boundary, i);
            boundary += 1;
        }
    }
    boundary
}
