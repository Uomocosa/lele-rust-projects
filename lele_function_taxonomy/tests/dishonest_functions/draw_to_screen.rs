static SCREEN: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());

pub fn draw_to_screen(_pixels: &[u8]) {
    SCREEN.lock().unwrap().extend_from_slice(_pixels);
}
