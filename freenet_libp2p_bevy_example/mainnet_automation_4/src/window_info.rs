#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: String,
    pub pid: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub title: String,
}
