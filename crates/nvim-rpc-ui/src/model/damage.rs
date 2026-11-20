use super::{Grid, GridId, State};

use std::sync::Arc;

/// A rectangular region of grid cells that needs repainting.
///
/// `top..bottom` and `left..right` are half-open cell ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridDamage {
    pub grid: GridId,
    pub top: usize,
    pub bottom: usize,
    pub left: usize,
    pub right: usize,
}

impl GridDamage {
    pub(super) fn whole(grid: GridId, content: &Grid) -> Self {
        Self {
            grid,
            top: 0,
            bottom: content.height,
            left: 0,
            right: content.width,
        }
    }

    fn clip(&mut self, state: &State) -> bool {
        let Some(grid) = state.grids.get(&self.grid) else {
            return false;
        };
        self.top = self.top.min(grid.height);
        self.bottom = self.bottom.min(grid.height);
        self.left = self.left.min(grid.width);
        self.right = self.right.min(grid.width);
        self.top < self.bottom && self.left < self.right
    }

    fn union(&mut self, other: &Self) {
        self.top = self.top.min(other.top);
        self.bottom = self.bottom.max(other.bottom);
        self.left = self.left.min(other.left);
        self.right = self.right.max(other.right);
    }
}

/// Repaint and layout changes between UI states.
///
/// An uncoalesced [`Frame::damage`](super::Frame::damage) describes changes since
/// the previous published frame. Coalescing combines pending frames' damage.
/// [`State::diff_since()`] computes damage against a snapshot selected by the
/// caller, including changes across skipped frames.
#[derive(Debug, Clone, Default)]
pub struct Damage {
    /// Changed cell regions, with at most one bounding rectangle per grid.
    pub grids: Vec<GridDamage>,
    /// Grids removed from the UI.
    ///
    /// A redraw can destroy a transient grid before its first resize. Such an
    /// ID need not be present in the preceding state.
    pub removed_grids: Vec<GridId>,
    /// Requests a repaint of the complete current state.
    pub full: bool,
    /// Grid sizes, placements, visibility, or viewports changed.
    pub layout: bool,
    /// The cursor position, style, or busy state changed.
    pub cursor: bool,
    /// External UI components or highlight-group mappings changed.
    pub widgets: bool,
}

impl Damage {
    pub(crate) fn record_grid(&mut self, region: GridDamage, max_grids: usize) {
        self.full |= !insert_region(&mut self.grids, region, max_grids);
    }

    pub(crate) fn record_removal(&mut self, id: GridId, max_grids: usize) {
        self.full |= !insert_removal(&mut self.removed_grids, id, max_grids);
        self.layout = true;
    }

    pub(super) fn merge(&mut self, next: Self, previous: &State, state: &State, max_grids: usize) {
        self.full |= next.full;
        self.layout |= next.layout;
        self.cursor |= next.cursor;
        self.widgets |= next.widgets;
        if !self.full {
            self.full = !self.merge_grids(next, previous, state, max_grids);
        }
        if self.full {
            self.grids.clear();
            self.removed_grids.clear();
        }
    }

    fn merge_grids(
        &mut self,
        next: Self,
        previous: &State,
        state: &State,
        max_grids: usize,
    ) -> bool {
        // Damage is public: clip and consolidate caller-supplied regions too.
        self.grids.retain_mut(|d| d.clip(state));
        self.grids.sort_unstable_by_key(|d| d.grid);
        self.grids.dedup_by(|a, b| {
            if a.grid != b.grid {
                return false;
            }
            b.union(a);
            true
        });
        if self.grids.len() > max_grids {
            return false;
        }
        for mut region in next.grids {
            if region.clip(state) && !merge_region(&mut self.grids, region, max_grids) {
                return false;
            }
        }

        // Removal followed by recreation can reuse an ID, even within one flush.
        // Such an ID needs a complete repaint, not a removal in the latest state.
        for id in self.removed_grids.iter().chain(&next.removed_grids) {
            if let Some(grid) = state.grids.get(id) {
                self.layout = true;
                if !merge_region(&mut self.grids, GridDamage::whole(*id, grid), max_grids) {
                    return false;
                }
            }
        }
        self.removed_grids
            .retain(|id| !state.grids.contains_key(id));
        self.removed_grids.sort_unstable();
        self.removed_grids.dedup();
        if self.removed_grids.len() > max_grids {
            return false;
        }
        for id in next.removed_grids {
            if !state.grids.contains_key(&id)
                && !insert_removal(&mut self.removed_grids, id, max_grids)
            {
                return false;
            }
        }

        if Arc::ptr_eq(&previous.grids, &state.grids) {
            return true;
        }
        for (&id, grid) in state.grids.iter() {
            if previous
                .grids
                .get(&id)
                .is_some_and(|old| old.width == grid.width && old.height == grid.height)
            {
                continue;
            }
            self.layout = true;
            if !merge_region(&mut self.grids, GridDamage::whole(id, grid), max_grids) {
                return false;
            }
        }
        for &id in previous.grids.keys() {
            if !state.grids.contains_key(&id) {
                self.layout = true;
                if !insert_removal(&mut self.removed_grids, id, max_grids) {
                    return false;
                }
            }
        }
        true
    }
}

fn insert_region(regions: &mut Vec<GridDamage>, region: GridDamage, max_grids: usize) -> bool {
    if let Some(old) = regions.iter_mut().find(|old| old.grid == region.grid) {
        old.union(&region);
    } else if regions.len() < max_grids {
        regions.push(region);
    } else {
        return false;
    }
    true
}

fn merge_region(regions: &mut Vec<GridDamage>, region: GridDamage, max_grids: usize) -> bool {
    // Coalescing discards empty regions; redraw recording retains event bounds.
    region.top >= region.bottom
        || region.left >= region.right
        || insert_region(regions, region, max_grids)
}

fn insert_removal(ids: &mut Vec<GridId>, id: GridId, max_grids: usize) -> bool {
    if !ids.contains(&id) {
        if ids.len() >= max_grids {
            return false;
        }
        ids.push(id);
    }
    true
}
