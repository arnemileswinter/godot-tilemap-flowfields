use std::collections::VecDeque;
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};

enum Dir {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Dir {
    fn offset(&self) -> (isize, isize) {
        use crate::algo::Dir::*;
        match self {
            North => (0, -1),
            East => (1, 0),
            South => (0, 1),
            West => (-1, 0),
            NorthEast => (1, -1),
            SouthEast => (1, 1),
            SouthWest => (-1, 1),
            NorthWest => (-1, -1),
        }
    }

    fn distance(&self) -> f32 {
        use crate::algo::Dir::*;
        match self {
            North | East | South | West => 1.,
            _ => SQRT_2,
        }
    }

    fn flow_vec(&self) -> Vector2D {
        use crate::algo::Dir::*;
        const V_N: (f32, f32) = (0., -1.);
        const V_NE: (f32, f32) = (FRAC_1_SQRT_2, -FRAC_1_SQRT_2);
        const V_E: (f32, f32) = (1., 0.);
        const V_SE: (f32, f32) = (FRAC_1_SQRT_2, FRAC_1_SQRT_2);
        const V_S: (f32, f32) = (0., 1.);
        const V_SW: (f32, f32) = (-FRAC_1_SQRT_2, FRAC_1_SQRT_2);
        const V_W: (f32, f32) = (-1., 0.);
        const V_NW: (f32, f32) = (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2);

        match self {
            North => V_N,
            East => V_E,
            South => V_S,
            West => V_W,
            NorthEast => V_NE,
            SouthEast => V_SE,
            SouthWest => V_SW,
            NorthWest => V_NW,
        }
    }
}

#[derive(
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    gdnative::prelude::FromVariant,
    gdnative::prelude::ToVariant,
)]
pub struct Dimensions {
    width: usize,
    height: usize,
    max_idx: usize,
}
impl Dimensions {
    pub fn new(width: usize, height: usize) -> Self {
        Dimensions {
            width,
            height,
            max_idx: (width * height),
        }
    }
    pub fn project_to_field_idx(&self, x: isize, y: isize) -> usize {
        (x + y * self.width as isize) as usize
    }
    pub fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize
    }
    pub fn max_idx(&self) -> usize {
        self.max_idx
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
}

pub type Vector2D = (f32, f32);
pub type Coord = (isize, isize);
pub type Cost = Option<f32>;
pub type CostField = Vec<Cost>;
pub type IntegrationField = Vec<Cost>;
pub type FlowField = Vec<Option<Vector2D>>;

pub fn calculate_integration_field(
    dim: &Dimensions,
    (to_x, to_y): Coord,
    cost_field: &CostField,
) -> Option<IntegrationField> {
    use crate::algo::Dir::*;
    assert_eq!(
        dim.max_idx(),
        cost_field.len(),
        "Cost field size does not match dimensions!"
    );
    let mut integration_field: IntegrationField = vec![None; dim.max_idx];
    let initial_cost = cost_field[dim.project_to_field_idx(to_x, to_y)];
    initial_cost?;
    integration_field[dim.project_to_field_idx(to_x, to_y)] = Some(0.);

    let mut queue: VecDeque<Coord> = vec![(to_x, to_y)].into();
    while !queue.is_empty() {
        match queue.pop_front() {
            None => (), // queue empty.
            Some((x, y)) => {
                let current_cost = integration_field[dim.project_to_field_idx(x, y)];
                if current_cost.is_none() {
                    continue;
                }

                let passable_at = |dir: Dir| {
                    let (off_x, off_y) = dir.offset();
                    let (x_next, y_next) = (x + off_x, y + off_y);
                    dim.in_bounds(x_next, y_next)
                        && cost_field[dim.project_to_field_idx(x_next, y_next)].is_some()
                };

                let mut visit = |dir: Dir| {
                    let (off_x, off_y) = dir.offset();
                    let (x_next, y_next) = (x + off_x, y + off_y);
                    if dim.in_bounds(x_next, y_next) {
                        let opt_new_cost = cost_field[dim.project_to_field_idx(x_next, y_next)]
                            .and_then(|c_static| {
                                current_cost.map(|c_acc| dir.distance() + c_static + c_acc)
                            });
                        let old_cost = integration_field[dim.project_to_field_idx(x_next, y_next)];
                        match (opt_new_cost, old_cost) {
                            (Some(n), Some(o)) if n < o => {
                                integration_field[dim.project_to_field_idx(x_next, y_next)] =
                                    opt_new_cost;
                                queue.push_back((x_next, y_next))
                            }
                            (Some(_), None) => {
                                integration_field[dim.project_to_field_idx(x_next, y_next)] =
                                    opt_new_cost;
                                queue.push_back((x_next, y_next))
                            }
                            _ => (),
                        }
                    }
                };

                if passable_at(North) && passable_at(East) {
                    visit(NorthEast);
                }
                if passable_at(South) && passable_at(East) {
                    visit(SouthEast);
                }
                if passable_at(South) && passable_at(West) {
                    visit(SouthWest);
                }
                if passable_at(North) && passable_at(West) {
                    visit(NorthWest);
                }
                visit(North);
                visit(East);
                visit(South);
                visit(West);
            }
        }
    }
    Some(integration_field)
}

pub fn calculate_flow_field(dim: &Dimensions, integration_field: &IntegrationField) -> FlowField {
    use crate::algo::Dir::*;
    assert_eq!(
        dim.max_idx(),
        integration_field.len(),
        "Integration field size does not match dimensions!"
    );
    let mut flow_field: FlowField = vec![None; dim.max_idx()];
    let integration_at = |x, y, dir: Dir| {
        let (x_off, y_off) = dir.offset();
        let (x_next, y_next) = (x + x_off, y + y_off);
        if dim.in_bounds(x_next, y_next) {
            integration_field[dim.project_to_field_idx(x_next, y_next)]
        } else {
            None
        }
    };

    /* in case a diagonal is walked, prevent flowing into impassable corners.
    Note this does not prevent walking diagonals in situations like: X_
                                                                     _X
    but only forces manhattan-like flow in situations like: __
                                                            _X
    where _ is a passable tile and X is an impassable tile. */
    let flow_without_corner_cutting =
        |(desired, desired_dir): (Option<f32>, Dir),
         (vertical_neighbor_integration, vertical_neighbor_dir): (Option<f32>, Dir),
         (horizontal_neighbor_integration, horizontal_neighbor_dir): (Option<f32>, Dir)| {
            desired.map(|_| {
                match (
                    vertical_neighbor_integration,
                    horizontal_neighbor_integration,
                ) {
                    (Some(_), Some(_)) | (None, None) => desired_dir.flow_vec(),
                    (_, Some(_)) => horizontal_neighbor_dir.flow_vec(),
                    (Some(_), _) => vertical_neighbor_dir.flow_vec(),
                }
            })
        };

    for x in 0..dim.width as isize {
        for y in 0..dim.height as isize {
            let current_vec = &mut flow_field[dim.project_to_field_idx(x, y)];
            if integration_field[dim.project_to_field_idx(x, y)].is_none() {
                *current_vec = None;
                continue;
            }
            let c_n = integration_at(x, y, North);
            let c_ne = integration_at(x, y, NorthEast);
            let c_e = integration_at(x, y, East);
            let c_se = integration_at(x, y, SouthEast);
            let c_s = integration_at(x, y, South);
            let c_sw = integration_at(x, y, SouthWest);
            let c_w = integration_at(x, y, West);
            let c_nw = integration_at(x, y, SouthWest);

            *current_vec = {
                let c = *[c_n, c_ne, c_e, c_se, c_s, c_sw, c_w, c_nw]
                    .iter()
                    .reduce(|last_lowest, i| match (i, last_lowest) {
                        (Some(c), Some(d)) if c < d => i,
                        (Some(_), None) => i,
                        _ => last_lowest,
                    })
                    .unwrap();
                /* given a desired flowing-cost, maps to the flow-field vector respecting passability. */
                let flow = |desired_dir: Dir| c.map(|_| desired_dir.flow_vec());
                if c == c_n {
                    flow(North)
                } else if c == c_ne {
                    flow_without_corner_cutting((c_ne, NorthEast), (c_n, North), (c_e, East))
                } else if c == c_e {
                    flow(East)
                } else if c == c_se {
                    flow_without_corner_cutting((c_se, SouthEast), (c_s, South), (c_e, East))
                } else if c == c_s {
                    flow(South)
                } else if c == c_sw {
                    flow_without_corner_cutting((c_sw, SouthWest), (c_s, South), (c_w, West))
                } else if c == c_w {
                    flow(West)
                } else if c == c_nw {
                    flow_without_corner_cutting((c_nw, NorthWest), (c_n, North), (c_w, West))
                } else {
                    unreachable!()
                }
            }
        }
    }
    flow_field
}

#[cfg(test)]
mod test {
    use crate::algo::*;
    use std::f32::consts::SQRT_2;

    #[test]
    fn small_integration_field() {
        let cost_field = vec![Some(0.); 9];
        let integration_field =
            crate::algo::calculate_integration_field(&Dimensions::new(3, 3), (1, 1), &cost_field);
        assert_eq!(
            integration_field,
            Some(vec![
                Some(SQRT_2),
                Some(1.),
                Some(SQRT_2),
                Some(1.),
                Some(0.),
                Some(1.),
                Some(SQRT_2),
                Some(1.),
                Some(SQRT_2)
            ]),
            "integration field doesn't match."
        )
    }
}
