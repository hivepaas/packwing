//! Where the window opens: where it was last left, if that is still on a screen.
//! As shellwing does it.
//!
//! Wayland does not let a client know or choose its position, so there only the
//! size and the maximized or fullscreen state come back; the compositor places
//! the window.

use gpui::{App, Bounds, Pixels, WindowBounds, point, px, size};

/// The size of a first window, and the least a window may be.
const DEFAULT_SIZE: (f32, f32) = (1280., 800.);
pub const MIN_SIZE: (f32, f32) = (900., 600.);

/// How much of the window must be on a display to keep its old position: enough
/// to grab the title bar.
const MIN_VISIBLE: (f32, f32) = (120., 40.);

/// How the window was left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowMode {
    Windowed,
    Maximized,
    Fullscreen,
}

/// The window's place and size as it comes back when it is no longer maximized,
/// in whole pixels, and how it was left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowPlace {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub mode: WindowMode,
}

impl WindowPlace {
    /// As the settings file has it: `x y width height mode`.
    pub fn to_text(self) -> String {
        let mode = match self.mode {
            WindowMode::Windowed => "windowed",
            WindowMode::Maximized => "maximized",
            WindowMode::Fullscreen => "fullscreen",
        };
        format!(
            "{} {} {} {} {mode}",
            self.x, self.y, self.width, self.height
        )
    }

    /// What [`WindowPlace::to_text`] wrote, if it is that.
    pub fn parse(text: &str) -> Option<WindowPlace> {
        let mut words = text.split(' ');
        let mut number = || words.next()?.parse().ok();
        let (x, y, width, height) = (number()?, number()?, number()?, number()?);
        let mode = match words.next()? {
            "windowed" => WindowMode::Windowed,
            "maximized" => WindowMode::Maximized,
            "fullscreen" => WindowMode::Fullscreen,
            _ => return None,
        };
        Some(WindowPlace {
            x,
            y,
            width,
            height,
            mode,
        })
    }
}

/// Where a new window goes.
pub fn initial(saved: Option<WindowPlace>, cx: &App) -> WindowBounds {
    let Some(s) = saved else {
        return WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(DEFAULT_SIZE.0), px(DEFAULT_SIZE.1)),
            cx,
        ));
    };
    let wanted = Bounds::new(
        point(px(s.x as f32), px(s.y as f32)),
        size(
            px((s.width as f32).max(MIN_SIZE.0)),
            px((s.height as f32).max(MIN_SIZE.1)),
        ),
    );
    let displays: Vec<Bounds<Pixels>> = cx.displays().iter().map(|d| d.bounds()).collect();
    let bounds = if on_a_display(wanted, &displays) {
        wanted
    } else {
        // The display it was on is gone: keep the size, center it.
        Bounds::centered(None, wanted.size, cx)
    };
    match s.mode {
        WindowMode::Windowed => WindowBounds::Windowed(bounds),
        WindowMode::Maximized => WindowBounds::Maximized(bounds),
        WindowMode::Fullscreen => WindowBounds::Fullscreen(bounds),
    }
}

fn on_a_display(window: Bounds<Pixels>, displays: &[Bounds<Pixels>]) -> bool {
    displays.iter().any(|d| {
        let v = d.intersect(&window).size;
        v.width >= px(MIN_VISIBLE.0) && v.height >= px(MIN_VISIBLE.1)
    })
}

/// What to keep of the window as it is.
pub fn place(bounds: WindowBounds) -> WindowPlace {
    let (b, mode) = match bounds {
        WindowBounds::Windowed(b) => (b, WindowMode::Windowed),
        WindowBounds::Maximized(b) => (b, WindowMode::Maximized),
        WindowBounds::Fullscreen(b) => (b, WindowMode::Fullscreen),
    };
    WindowPlace {
        x: f32::from(b.origin.x).round() as i32,
        y: f32::from(b.origin.y).round() as i32,
        width: f32::from(b.size.width).round() as i32,
        height: f32::from(b.size.height).round() as i32,
        mode,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
    }

    #[test]
    fn keeps_a_position_only_on_a_display() {
        let screens = [b(0., 0., 1920., 1080.), b(1920., 0., 2560., 1440.)];
        assert!(
            on_a_display(b(2000., 100., 1000., 700.), &screens),
            "second screen"
        );
        assert!(
            on_a_display(b(1800., 100., 1000., 700.), &screens),
            "across both"
        );
        // The second screen was unplugged.
        assert!(!on_a_display(b(2000., 100., 1000., 700.), &screens[..1]));
        // Only a sliver left on screen.
        assert!(!on_a_display(b(1900., 100., 1000., 700.), &screens[..1]));
    }

    #[test]
    fn it_keeps_the_size_to_restore_to_and_the_mode() {
        let kept = place(WindowBounds::Maximized(b(10., 20., 1200., 800.)));
        assert_eq!(
            kept,
            WindowPlace {
                x: 10,
                y: 20,
                width: 1200,
                height: 800,
                mode: WindowMode::Maximized,
            }
        );
    }

    #[test]
    fn it_reads_what_it_wrote_and_nothing_else() {
        let left = WindowPlace {
            x: -40,
            y: 20,
            width: 1400,
            height: 900,
            mode: WindowMode::Fullscreen,
        };
        assert_eq!(WindowPlace::parse(&left.to_text()), Some(left));
        for bad in ["", "1 2 3", "1 2 3 4", "1 2 3 4 huge", "a 2 3 4 windowed"] {
            assert_eq!(WindowPlace::parse(bad), None, "{bad:?}");
        }
    }
}
