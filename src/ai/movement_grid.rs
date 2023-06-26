use itertools::Itertools;

use std::collections::{HashMap, VecDeque};

use crate::{physics::*, prelude::Transform};

#[derive(Clone, Debug)]
pub struct MovementGrid {
    cell_size: isize,
    from: (f32, f32),
    to: (f32, f32),
    free_cells: HashMap<(isize, isize), bool>,
}

impl MovementGrid {
    fn pos_to_cell_idx(cell_size: isize, pos: &(f32, f32)) -> (isize, isize) {
        let cell_row = (pos.0 / cell_size as f32).floor() as isize;
        let cell_col = (pos.1 / cell_size as f32).floor() as isize;
        (cell_row, cell_col)
    }

    pub fn new(cell_size: isize, from: &(f32, f32), to: &(f32, f32)) -> Self {
        let top_left = (from.0.min(to.0), from.1.min(to.1));
        let btm_right = (from.0.max(to.0), from.1.max(to.1));
        let top_left_cell_idx = Self::pos_to_cell_idx(cell_size, &top_left);
        let btm_right_cell_idx = Self::pos_to_cell_idx(cell_size, &btm_right);
        let (min_x, max_x) = (top_left_cell_idx.0, btm_right_cell_idx.0);
        let (min_y, max_y) = (top_left_cell_idx.1, btm_right_cell_idx.1);
        let mut cell_obstructions = HashMap::new();
        for row in min_y..=max_y {
            for col in min_x..=max_x {
                cell_obstructions.insert((col, row), true);
            }
        }
        Self {
            cell_size,
            from: *from,
            to: *to,
            free_cells: cell_obstructions,
        }
    }

    fn free_successors(&self, pos: &(isize, isize)) -> impl IntoIterator<Item = (isize, isize)> {
        let top = (pos.0, pos.1 + 1);
        let btm = (pos.0, pos.1 - 1);
        let right = (pos.0 + 1, pos.1);
        let left = (pos.0 - 1, pos.1);
        let perp_successors = [top, btm, right, left]
            .into_iter()
            .filter(|pos| self.is_free(pos))
            .collect_vec();
        let mut diagonal_successors = Vec::new();
        let top_right = (pos.0 + 1, pos.1 + 1);
        let btm_right = (pos.0 + 1, pos.1 - 1);
        let top_left = (pos.0 - 1, pos.1 + 1);
        let btm_left = (pos.0 - 1, pos.1 - 1);
        if self.is_free(&top_right) && (self.is_free(&top) && self.is_free(&right)) {
            diagonal_successors.push(top_right);
        }
        if self.is_free(&top_left) && (self.is_free(&top) && self.is_free(&left)) {
            diagonal_successors.push(top_left);
        }
        if self.is_free(&btm_right) && (self.is_free(&btm) && self.is_free(&right)) {
            diagonal_successors.push(btm_right);
        }
        if self.is_free(&btm_left) && (self.is_free(&btm) && self.is_free(&left)) {
            diagonal_successors.push(btm_left);
        }
        perp_successors
            .into_iter()
            .chain(diagonal_successors.into_iter())
    }

    pub fn find_path(&self) -> Option<VecDeque<(f32, f32)>> {
        let src = Self::pos_to_cell_idx(self.cell_size, &self.from);
        let dst = Self::pos_to_cell_idx(self.cell_size, &self.to);
        if !self.is_free(&dst) {
            return None;
        }
        if !self.is_free(&src) {
            return None;
        }
        let astar_path = pathfinding::prelude::astar(
            &src,
            |cell_idx| {
                let successors = self
                    .free_successors(cell_idx)
                    .into_iter()
                    .filter(|cell_idx| self.is_free(cell_idx))
                    .collect_vec();
                let cell_idx = *cell_idx;
                successors.into_iter().map(move |succ| {
                    let dx = cell_idx.0 - succ.0;
                    let dy = cell_idx.1 - succ.1;
                    (succ, dx.abs() + dy.abs())
                })
            },
            |cell_idx| {
                let dx = cell_idx.0 - dst.0;
                let dy = cell_idx.1 - dst.1;
                dx.abs() + dy.abs()
            },
            |cell_idx| *cell_idx == dst,
        );
        astar_path.map(|(output, _)| {
            VecDeque::from_iter(output.into_iter().map(|pos| {
                (
                    (pos.0 * self.cell_size) as f32 + self.cell_size as f32 / 2.,
                    (pos.1 * self.cell_size) as f32 + self.cell_size as f32 / 2.,
                )
            }))
        })
    }

    pub fn fill_obstructions(&mut self, hitboxes: &Vec<EffectiveHitbox>) {
        self.free_cells.iter_mut().for_each(|(cell_idx, free)| {
            let transform = Transform::at(
                (cell_idx.0 * self.cell_size) as f32 + self.cell_size as f32 / 2.,
                (cell_idx.1 * self.cell_size) as f32 + self.cell_size as f32 / 2.,
            );
            let shape = Shape::Rect {
                w: self.cell_size as f32,
                h: self.cell_size as f32,
            };
            let cell_hb = TransformedShape::new(&transform, &shape, (0., 0.));
            let is_colliding = hitboxes.iter().any(|target_hb| {
                sepax2d::sat_overlap(target_hb.shape.shape_ref(), cell_hb.shape_ref())
            });
            *free = !is_colliding;
        });
    }

    fn is_free(&self, cell_idx: &(isize, isize)) -> bool {
        self.free_cells.get(&cell_idx).map(|v| *v).unwrap_or(false)
    }
}
