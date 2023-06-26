#[derive(Debug)]
pub struct Rect {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

impl Rect {
    pub fn from_win32(rect: windows::Win32::Foundation::RECT) -> Self {
        Self {
            left: rect.left,
            right: rect.right,
            top: rect.top,
            bottom: rect.bottom,
        }
    }
}

#[derive(Debug)]
pub struct Monitor {
    pub title: String,
    pub id: String,
    pub gdi_name: String,
    pub rect: Rect,
}
impl Monitor {
    pub fn from_win32(
        title: String,
        id: String,
        gdi_name: String,
        rect: windows::Win32::Foundation::RECT,
    ) -> Self {
        Self {
            title,
            id,
            gdi_name,
            rect: Rect::from_win32(rect),
        }
    }
}
