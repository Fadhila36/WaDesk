use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::error::Result;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowPlacement, SetWindowPlacement, GetWindowRect, IsZoomed,
    SW_MAXIMIZE, SW_SHOWNORMAL, WINDOWPLACEMENT,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: i32,
    pub height: i32,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 1100,
            height: 750,
            maximized: false,
        }
    }
}

impl WindowState {
    pub fn load_or_default(path: &Path) -> Self {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(state) = serde_json::from_slice::<WindowState>(&bytes) {
                return state;
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn from_hwnd(hwnd: HWND) -> Self {
        unsafe {
            let mut wp = WINDOWPLACEMENT {
                length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };

            let maximized = IsZoomed(hwnd).as_bool();

            if GetWindowPlacement(hwnd, &mut wp).is_ok() {
                let rect = wp.rcNormalPosition;
                let width = (rect.right - rect.left).max(400);
                let height = (rect.bottom - rect.top).max(300);

                // Abaikan koordinat -32000 jika jendela sedang diminimalkan ke tray
                let (x, y) = if rect.left > -10000 && rect.top > -10000 {
                    (Some(rect.left), Some(rect.top))
                } else {
                    (None, None)
                };

                Self {
                    x,
                    y,
                    width,
                    height,
                    maximized,
                }
            } else {
                let mut rect = RECT::default();
                let _ = GetWindowRect(hwnd, &mut rect);

                let width = (rect.right - rect.left).max(400);
                let height = (rect.bottom - rect.top).max(300);

                let (x, y) = if rect.left > -10000 && rect.top > -10000 {
                    (Some(rect.left), Some(rect.top))
                } else {
                    (None, None)
                };

                Self {
                    x,
                    y,
                    width,
                    height,
                    maximized,
                }
            }
        }
    }

    pub fn apply_to_hwnd(&self, hwnd: HWND) {
        unsafe {
            let mut wp = WINDOWPLACEMENT {
                length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };

            if GetWindowPlacement(hwnd, &mut wp).is_ok() {
                if let (Some(x), Some(y)) = (self.x, self.y) {
                    wp.rcNormalPosition = RECT {
                        left: x,
                        top: y,
                        right: x + self.width,
                        bottom: y + self.height,
                    };
                }

                wp.showCmd = if self.maximized {
                    SW_MAXIMIZE.0 as u32
                } else {
                    SW_SHOWNORMAL.0 as u32
                };

                let _ = SetWindowPlacement(hwnd, &wp);
            }
        }
    }
}
