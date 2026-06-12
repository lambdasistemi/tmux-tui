//! Pure geometry: scaling tmux cell-coordinates into screen rectangles and
//! hit-testing the cursor against them. No tmux or terminal I/O here.

use ratatui::layout::Rect;

use crate::tmux::Pane;

/// Scale a pane's tmux cell-geometry into the on-screen `area`.
pub fn map_rect(p: &Pane, area: Rect, win_w: u16, win_h: u16) -> Rect {
    let sx = area.width as f32 / win_w.max(1) as f32;
    let sy = area.height as f32 / win_h.max(1) as f32;
    let x = (area.x + (p.left as f32 * sx).round() as u16).min(area.right().saturating_sub(1));
    let y = (area.y + (p.top as f32 * sy).round() as u16).min(area.bottom().saturating_sub(1));
    let max_w = area.right().saturating_sub(x).max(1);
    let max_h = area.bottom().saturating_sub(y).max(1);
    let w = ((p.width as f32 * sx).round() as u16).clamp(1, max_w);
    let h = ((p.height as f32 * sy).round() as u16).clamp(1, max_h);
    Rect { x, y, width: w, height: h }
}

pub fn rects_for(panes: &[Pane], canvas: Rect, win_w: u16, win_h: u16) -> Vec<(usize, Rect)> {
    panes.iter().enumerate().map(|(i, p)| (i, map_rect(p, canvas, win_w, win_h))).collect()
}

pub fn contains(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.right() && row >= r.y && row < r.bottom()
}

/// Pane under the cursor; the smallest containing box wins so borders never
/// block selection of an inner pane.
pub fn hit(rects: &[(usize, Rect)], col: u16, row: u16) -> Option<usize> {
    rects
        .iter()
        .filter(|(_, r)| contains(*r, col, row))
        .min_by_key(|(_, r)| r.width as u32 * r.height as u32)
        .map(|(i, _)| *i)
}

/// Window id of the tile under the cursor (window mode).
pub fn tile_at(tiles: &[(String, Rect)], col: u16, row: u16) -> Option<String> {
    tiles.iter().find(|(_, r)| contains(*r, col, row)).map(|(id, _)| id.clone())
}

/// (window id, pane id) of the mini-pane under the cursor (window mode).
pub fn mpane_at(mpanes: &[(String, String, Rect)], col: u16, row: u16) -> Option<(String, String)> {
    mpanes
        .iter()
        .filter(|(_, _, r)| contains(*r, col, row))
        .min_by_key(|(_, _, r)| r.width as u32 * r.height as u32)
        .map(|(w, p, _)| (w.clone(), p.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::Pane;

    fn pane(left: u16, top: u16, width: u16, height: u16) -> Pane {
        Pane {
            id: "%0".into(),
            index: "0".into(),
            left,
            top,
            width,
            height,
            active: false,
            cmd: "sh".into(),
        }
    }

    #[test]
    fn contains_is_half_open() {
        let r = Rect { x: 2, y: 3, width: 4, height: 2 };
        assert!(contains(r, 2, 3));
        assert!(contains(r, 5, 4));
        assert!(!contains(r, 6, 3)); // x == right() is outside
        assert!(!contains(r, 2, 5)); // y == bottom() is outside
    }

    #[test]
    fn map_rect_fills_canvas_when_pane_fills_window() {
        let canvas = Rect { x: 0, y: 0, width: 80, height: 24 };
        let m = map_rect(&pane(0, 0, 100, 50), canvas, 100, 50);
        assert_eq!(m, canvas);
    }

    #[test]
    fn map_rect_scales_a_right_half_pane() {
        let canvas = Rect { x: 0, y: 0, width: 80, height: 24 };
        let m = map_rect(&pane(50, 0, 50, 50), canvas, 100, 50);
        assert_eq!(m, Rect { x: 40, y: 0, width: 40, height: 24 });
    }

    #[test]
    fn hit_prefers_the_smallest_containing_box() {
        let rects = [
            (0usize, Rect { x: 0, y: 0, width: 10, height: 10 }),
            (1usize, Rect { x: 2, y: 2, width: 3, height: 3 }),
        ];
        assert_eq!(hit(&rects, 3, 3), Some(1));
        assert_eq!(hit(&rects, 0, 0), Some(0));
        assert_eq!(hit(&rects, 50, 50), None);
    }
}
