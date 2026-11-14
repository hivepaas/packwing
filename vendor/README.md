# Crates patched for packwing

Published crates packwing builds with, changed for packwing, and put in place
of the published ones by `[patch.crates-io]` in the workspace's `Cargo.toml`.
Each is the published crate as it was, with its own license, and changes marked
`packwing:` in the code. Moving to a newer version of one means copying the
new version here and making the changes again, or dropping it once upstream has
what packwing needs.

## gpui-pre-macos 0.3.9

GPUI's macOS platform (Zed's `gpui_macos`, Apache-2.0). Its `app-store`
feature, which packwing's `app-store` feature turns on, leaves out what AppKit
keeps private, as the Mac App Store refuses apps that use it (App Review
Guideline 2.5.1); every other build is as published.

| Private, as published | In the store's build |
| --- | --- |
| `_opaqueRectForWindowMoveWhenInTitlebar`, overridden on GPUI's view: a window that drags itself from its titlebar | not overridden: AppKit drags the window, as for every window packwing opens |
| `_windowRestorationOptions`, overridden on GPUI's unarchiver (and the class `NSWindowRestorationOptions`): a restored window goes back to its Space on macOS 15+ | not overridden: packwing restores no window state |
| `-[NSWindow _zoomFill:]`, the titlebar's double-click set to Fill | `zoom:` |
| `+[NSCursor _windowResizeNorthWestSouthEastCursor]` and `…NorthEastSouthWest…` | `+[NSCursor frameResizeCursorFromPosition:inDirections:]` (macOS 15), the arrow before |

`scripts/package_macos_appstore.sh` checks that the store's program names none
of them.
