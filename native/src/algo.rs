use std::collections::VecDeque;
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};

#[derive(Clone, Copy)]
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
) -> IntegrationField {
    assert_eq!(
        dim.max_idx(),
        cost_field.len(),
        "Cost field size does not match dimensions!"
    );
    let mut integration_field: IntegrationField = vec![None; dim.max_idx];
    let initial_cost = cost_field[dim.project_to_field_idx(to_x, to_y)];
    integration_field[dim.project_to_field_idx(to_x, to_y)] = initial_cost;
    if initial_cost.is_none() {
        // trying to path-find to impassable location.
        // results in the None-field as is.
        return integration_field;
    }
    let mut queue: VecDeque<Coord> = vec![(to_x, to_y)].into();
    while !queue.is_empty() {
        match queue.pop_front() {
            None => (), // queue empty.
            Some((x, y)) => {
                let current_cost = cost_field[dim.project_to_field_idx(x, y)];
                if current_cost.is_none() {
                    continue;
                }

                let mut visit = |off_x: isize, off_y: isize, distance_cost: f32| {
                    let (x_next, y_next) = (x + off_x, y + off_y);
                    if dim.in_bounds(x_next, y_next) {
                        let opt_new_cost = cost_field[dim.project_to_field_idx(x_next, y_next)]
                            .and_then(|c_static| {
                                current_cost.map(|c_acc| distance_cost + c_static + c_acc)
                            });
                        let old_cost = integration_field[dim.project_to_field_idx(x_next, y_next)];
                        match (opt_new_cost, old_cost) {
                            (Some(n), Some(o)) => {
                                if n < o {
                                    integration_field[dim.project_to_field_idx(x_next, y_next)] =
                                        opt_new_cost;
                                    queue.push_back((x_next, y_next))
                                }
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

                visit(0, -1, 1.0); // north
                visit(1, -1, SQRT_2); // north-east
                visit(1, 0, 0.0); // east
                visit(1, 1, SQRT_2); // south-east
                visit(0, 1, 1.0); // south
                visit(-1, 1, SQRT_2); // south-west
                visit(-1, 0, 1.0); // west
                visit(-1, -1, SQRT_2); // north-west
            }
        }
    }
    integration_field
}

pub fn calculate_flow_field(dim: &Dimensions, integration_field: &IntegrationField) -> FlowField {
    assert_eq!(
        dim.max_idx(),
        integration_field.len(),
        "Integration field size does not match dimensions!"
    );
    let mut flow_field: FlowField = vec![None; dim.max_idx];
    let integration_at = |x, y| {
        if dim.in_bounds(x, y) {
            integration_field[dim.project_to_field_idx(x, y)]
        } else {
            None
        }
    };
    for x in 0..dim.width as isize {
        for y in 0..dim.height as isize {
            let c_n = integration_at(x, y - 1);
            let c_ne = integration_at(x + 1, y - 1);
            let c_e = integration_at(x + 1, y);
            let c_se = integration_at(x + 1, y + 1);
            let c_s = integration_at(x, y + 1);
            let c_sw = integration_at(x - 1, y + 1);
            let c_w = integration_at(x - 1, y);
            let c_nw = integration_at(x - 1, y - 1);

            flow_field[dim.project_to_field_idx(x, y)] =
                match [c_n, c_ne, c_e, c_se, c_s, c_sw, c_w, c_nw]
                    .iter()
                    .reduce(|m, i| if i < m { m } else { i })
                    .unwrap()
                {
                    m if m == &c_n => c_n.map(|_| (0., -1.)),
                    m if m == &c_ne => c_ne.map(|_| (FRAC_1_SQRT_2, -FRAC_1_SQRT_2)),
                    m if m == &c_e => c_e.map(|_| (1., 0.)),
                    m if m == &c_se => c_se.map(|_| (FRAC_1_SQRT_2, FRAC_1_SQRT_2)),
                    m if m == &c_s => c_s.map(|_| (0., 1.)),
                    m if m == &c_sw => c_sw.map(|_| (-FRAC_1_SQRT_2, FRAC_1_SQRT_2)),
                    m if m == &c_w => c_w.map(|_| (-1., 0.)),
                    m if m == &c_nw => c_nw.map(|_| (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2)),
                    _ => unreachable!(),
                };
        }
    }
    flow_field
}
