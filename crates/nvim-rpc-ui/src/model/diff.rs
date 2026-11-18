use super::{Damage, Grid, GridDamage, GridId, State};

use std::sync::Arc;

impl State {
    /// Computes repaint damage relative to a previously rendered snapshot.
    ///
    /// Keep the state from the frame your backend last painted, then compare
    /// the latest state with it. Intermediate frames need not be compared or
    /// retained. Grid removals and layout, highlight, option, cursor, and widget
    /// changes are included.
    ///
    /// The result is conservative: unchanged shared storage is skipped, while
    /// independently allocated data may be reported as changed even when its
    /// contents are equal. Comparisons are valid between unrelated states too;
    /// no sequence or connection identity is required.
    ///
    /// This method does not recover events from skipped frames. Handle every
    /// frame's events in order, independently of the snapshots used for paint.
    ///
    /// ```
    /// use nvim_rpc_ui::State;
    ///
    /// let previous = State::default();
    /// let current = State { busy: true, ..previous.clone() };
    /// let damage = current.diff_since(&previous);
    /// assert!(damage.cursor);
    /// assert!(!damage.full);
    /// assert!(damage.grids.is_empty());
    /// ```
    pub fn diff_since(&self, previous: &Self) -> Damage {
        if std::ptr::eq(self, previous) {
            return Damage::default();
        }

        let cursor_changed = match (self.cursor, previous.cursor) {
            (None, None) => false,
            (Some(a), Some(b)) => a.grid != b.grid || a.row != b.row || a.col != b.col,
            _ => true,
        };
        let colors_changed = self.default_colors.foreground != previous.default_colors.foreground
            || self.default_colors.background != previous.default_colors.background
            || self.default_colors.special != previous.default_colors.special
            || self.default_colors.cterm_foreground != previous.default_colors.cterm_foreground
            || self.default_colors.cterm_background != previous.default_colors.cterm_background;
        let mut damage = Damage {
            full: colors_changed
                || !Arc::ptr_eq(&self.highlights, &previous.highlights)
                || !Arc::ptr_eq(&self.options, &previous.options)
                || self.title != previous.title
                || self.icon != previous.icon
                || self.cwd != previous.cwd
                || self.mouse_enabled != previous.mouse_enabled,
            layout: !Arc::ptr_eq(&self.placements, &previous.placements)
                || !Arc::ptr_eq(&self.viewports, &previous.viewports),
            cursor: cursor_changed
                || self.busy != previous.busy
                || self.mode.name != previous.mode.name
                || self.mode.index != previous.mode.index
                || self.mode.cursor_style_enabled != previous.mode.cursor_style_enabled
                || !Arc::ptr_eq(&self.modes, &previous.modes),
            widgets: !Arc::ptr_eq(&self.highlight_groups, &previous.highlight_groups)
                || !Arc::ptr_eq(&self.cmdlines, &previous.cmdlines)
                || !Arc::ptr_eq(&self.cmdline_block, &previous.cmdline_block)
                || !shares_optional(&self.popupmenu, &previous.popupmenu)
                || !shares_optional(&self.tabline, &previous.tabline)
                || !shares_optional(&self.wildmenu, &previous.wildmenu)
                || !Arc::ptr_eq(&self.messages, &previous.messages),
            ..Damage::default()
        };

        if !Arc::ptr_eq(&self.grids, &previous.grids) {
            for (&id, grid) in self.grids.iter() {
                let region = match previous.grids.get(&id) {
                    Some(old) if Arc::ptr_eq(grid, old) => continue,
                    Some(old) => {
                        damage.layout |= grid.width != old.width || grid.height != old.height;
                        grid_damage(id, grid, old)
                    }
                    None => {
                        damage.layout = true;
                        Some(GridDamage::whole(id, grid))
                    }
                };
                if let Some(region) = region {
                    damage.grids.push(region);
                }
            }
            for &id in previous.grids.keys() {
                if !self.grids.contains_key(&id) {
                    damage.removed_grids.push(id);
                    damage.layout = true;
                }
            }
        }
        damage
    }
}

fn shares_optional<T>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}

fn grid_damage(id: GridId, grid: &Grid, previous: &Grid) -> Option<GridDamage> {
    if grid.width != previous.width
        || grid.height != previous.height
        || grid.rows.len() != previous.rows.len()
        || grid.rows.len() != grid.height
        || previous.rows.len() != previous.height
    {
        return Some(GridDamage::whole(id, grid));
    }
    let mut changed = None;
    for (idx, (row, old)) in grid.rows.iter().zip(&previous.rows).enumerate() {
        if !Arc::ptr_eq(row, old) {
            let top = changed.map_or(idx, |(top, _)| top);
            changed = Some((top, idx.saturating_add(1).min(grid.height)));
        }
    }
    changed.map(|(top, bottom)| GridDamage {
        grid: id,
        top,
        bottom,
        left: 0,
        right: grid.width,
    })
}
