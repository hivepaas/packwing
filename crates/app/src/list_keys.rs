//! The keys that move the chosen row of a list, as file managers take them:
//! Up and Down a row, Page Up and Page Down a page, Home and End to the first
//! and the last. Browse and the Archive page move alike.

use gpui::UniformListScrollHandle;

/// The rows Page Up and Page Down move by before a list is drawn.
const PAGE_BEFORE_DRAWN: usize = 10;

/// How many rows Page Up and Page Down move by in the list of `len` rows
/// `scroll` scrolls: as many as are in sight, but one, so that the row chosen
/// last stays in sight; [`PAGE_BEFORE_DRAWN`] until the list is drawn.
pub fn rows_per_page(scroll: &UniformListScrollHandle, len: usize) -> usize {
    // What GPUI keeps of the last layout: the list's own size, in sight, as
    // its "item", and all its rows' as its contents.
    let Some(sizes) = scroll.0.borrow().last_item_size else {
        return PAGE_BEFORE_DRAWN;
    };
    page_of(sizes.item.height.into(), sizes.contents.height.into(), len)
}

/// The rows in sight but one, from the height in sight and all `len` rows'.
fn page_of(seen: f32, all: f32, len: usize) -> usize {
    if len == 0 || seen <= 0. || all <= 0. {
        return PAGE_BEFORE_DRAWN;
    }
    let row = all / len as f32;
    ((seen / row).floor() as usize).saturating_sub(1).max(1)
}

/// The row of a list of `len` that `key` moves the choice to, from the one
/// chosen at `from` (none chosen: Down and Page Down choose the first, Up and
/// Page Up the last), a page being `page` rows; `None` if `key` moves nothing.
pub fn moved(key: &str, from: Option<usize>, len: usize, page: usize) -> Option<usize> {
    let last = len.checked_sub(1)?;
    let page = page.max(1);
    Some(
        match (key, from) {
            ("home", _) => 0,
            ("end", _) => last,
            ("down" | "pagedown", None) => 0,
            ("up" | "pageup", None) => last,
            ("down", Some(f)) => (f + 1).min(last),
            ("up", Some(f)) => f.saturating_sub(1),
            ("pagedown", Some(f)) => f.saturating_add(page).min(last),
            ("pageup", Some(f)) => f.saturating_sub(page),
            _ => return None,
        }
        .min(last),
    )
}

#[cfg(test)]
mod tests {
    use super::{PAGE_BEFORE_DRAWN, moved, page_of};

    #[test]
    fn a_page_is_what_is_in_sight_but_a_row() {
        // 31 rows of 48, 500 in sight: 10 whole rows.
        assert_eq!(page_of(500., 31. * 48., 31), 9);
        assert_eq!(page_of(30., 31. * 48., 31), 1);
        assert_eq!(page_of(500., 0., 0), PAGE_BEFORE_DRAWN);
    }

    #[test]
    fn keys_move_the_choice() {
        assert_eq!(moved("down", Some(3), 10, 4), Some(4));
        assert_eq!(moved("down", Some(9), 10, 4), Some(9));
        assert_eq!(moved("up", Some(0), 10, 4), Some(0));
        assert_eq!(moved("pagedown", Some(3), 10, 4), Some(7));
        assert_eq!(moved("pagedown", Some(8), 10, 4), Some(9));
        assert_eq!(moved("pageup", Some(3), 10, 4), Some(0));
        assert_eq!(moved("pageup", Some(9), 10, 4), Some(5));
        assert_eq!(moved("home", Some(5), 10, 4), Some(0));
        assert_eq!(moved("end", None, 10, 4), Some(9));
        // Nothing chosen: down from the top, up from the bottom.
        assert_eq!(moved("down", None, 10, 4), Some(0));
        assert_eq!(moved("pageup", None, 10, 4), Some(9));
        // A chosen row gone with a shorter list stays in it.
        assert_eq!(moved("up", Some(20), 10, 4), Some(9));
        assert_eq!(moved("down", Some(0), 0, 4), None);
        assert_eq!(moved("left", Some(0), 10, 4), None);
    }
}
