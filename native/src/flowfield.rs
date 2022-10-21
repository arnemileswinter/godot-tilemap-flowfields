use gdnative::api::Resource;
use gdnative::prelude::*;

use crate::algo::{self, Dimensions};

fn round_vec(v: Vector2) -> Result<(isize, isize), GodotString> {
    if v.x.is_nan() || v.x.is_infinite() || v.y.is_nan() || v.y.is_infinite() {
        Err(format!("Bad vector access: {}", v.to_variant()).into())
    } else {
        Ok((v.x as isize, v.y as isize))
    }
}

trait HasDim {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn mut_dim(&mut self) -> &mut algo::Dimensions;

    fn recalculate_dim(&mut self) {
        *self.mut_dim() = algo::Dimensions::new(self.width(), self.height())
    }
}

pub struct FlowFieldFactory {}
impl FlowFieldFactory {
    pub fn create(dim: algo::Dimensions, field: algo::FlowField) -> FlowField {
        FlowField {
            dim,
            width: dim.width() as u64,
            height: dim.height() as u64,
            field,
        }
    }
}

#[derive(NativeClass)]
#[inherit(Resource)]
#[register_with(Self::register_properties)]
pub struct FlowField {
    dim: algo::Dimensions,
    #[property]
    width: u64,
    #[property]
    height: u64,
    #[property]
    field: algo::FlowField,
}

impl HasDim for FlowField {
    fn width(&self) -> usize {
        self.width as usize
    }
    fn height(&self) -> usize {
        self.height as usize
    }
    fn mut_dim(&mut self) -> &mut algo::Dimensions {
        &mut self.dim
    }
}

impl FlowField {
    fn can_flow_internal(&self, (from_x, from_y): algo::Coord) -> bool {
        self.dim.in_bounds(from_x, from_y)
            && self.field[self.dim.project_to_field_idx(from_x, from_y)].is_some()
    }

    fn flow_internal(
        &self,
        from @ (from_x, from_y): algo::Coord,
    ) -> Result<algo::Vector2D, String> {
        if !self.dim.in_bounds(from_x, from_y) {
            Err(format!("FlowField: position {:#?} out of bounds!", from))
        } else if let Some((vx, vy)) = self.field[self.dim.project_to_field_idx(from_x, from_y)] {
            Ok((vx, vy))
        } else {
            Err(format!(
                "FlowField: unreachable position {:#?} queried!",
                from
            ))
        }
    }
}

#[methods]
impl FlowField {
    fn new(_owner: &Resource) -> Self {
        Self {
            dim: algo::Dimensions::new(0, 0),
            width: 0,
            height: 0,
            field: vec![],
        }
    }

    fn register_properties(builder: &ClassBuilder<FlowField>) {
        builder
            .property("field")
            .with_getter(|s, _| s.field.to_owned())
            .with_setter(|s: &mut Self, _, new_val: algo::FlowField| s.field = new_val)
            .with_default(vec![])
            .done();
        builder
            .property("width")
            .with_getter(|s: &Self, _| s.width)
            .with_setter(|s: &mut Self, _, new_val| {
                s.width = new_val;
                s.recalculate_dim()
            })
            .with_default(0)
            .done();
        builder
            .property("height")
            .with_getter(|s: &Self, _| s.height)
            .with_setter(|s: &mut Self, _, new_val| {
                s.height = new_val;
                s.recalculate_dim()
            })
            .with_default(0)
            .done();
    }

    #[method]
    fn can_flow(&self, #[base] _owner: TRef<'_, Resource>, from: Vector2) -> bool {
        match round_vec(from) {
            Err(msg) => {
                godot_error!("FlowField: {}", msg);
                false
            }
            Ok((x, y)) => {
                self.dim.in_bounds(x, y)
                    && self.field[self.dim.project_to_field_idx(x, y)].is_some()
            }
        }
    }

    #[method]
    fn flow(&self, #[base] _owner: TRef<'_, Resource>, from: Vector2) -> Vector2 {
        match round_vec(from) {
            Err(msg) => {
                godot_error!("FlowField: {}", msg);
            }
            Ok((x, y)) => match self.flow_internal((x, y)) {
                Err(m) => godot_error!("FlowField: {}", m),
                Ok((vx, vy)) => return Vector2 { x: vx, y: vy },
            },
        }
        Vector2::ZERO
    }
}

pub struct BakedFlowFieldsFactory {}
impl BakedFlowFieldsFactory {
    pub fn create(dim: Dimensions, fields: Vec<Instance<FlowField>>) -> BakedFlowFields {
        BakedFlowFields {
            dim,
            width: dim.width() as u64,
            height: dim.height() as u64,
            flow_fields: fields,
        }
    }
}

#[derive(NativeClass)]
#[inherit(Resource)]
#[register_with(Self::register_properties)]
pub struct BakedFlowFields {
    dim: Dimensions,
    #[property]
    width: u64,
    #[property]
    height: u64,
    #[property]
    flow_fields: Vec<Instance<FlowField>>,
}
impl HasDim for BakedFlowFields {
    fn width(&self) -> usize {
        self.width as usize
    }
    fn height(&self) -> usize {
        self.height as usize
    }
    fn mut_dim(&'_ mut self) -> &'_ mut algo::Dimensions {
        &mut self.dim
    }
}
#[methods]
impl BakedFlowFields {
    fn new(_owner: &Resource) -> Self {
        Self {
            dim: algo::Dimensions::new(0, 0),
            width: 0,
            height: 0,
            flow_fields: vec![],
        }
    }

    fn register_properties(builder: &ClassBuilder<BakedFlowFields>) {
        builder
            .property("field")
            .with_getter(|s, _| s.flow_fields.to_owned())
            .with_setter(|s: &mut Self, _, new_val: Vec<Instance<FlowField>>| {
                s.flow_fields = new_val
            })
            .with_default(vec![])
            .done();
        builder
            .property("width")
            .with_getter(|s: &Self, _| s.width)
            .with_setter(|s: &mut Self, _, new_val| {
                s.width = new_val;
                s.recalculate_dim()
            })
            .with_default(0)
            .done();
        builder
            .property("height")
            .with_getter(|s: &Self, _| s.height)
            .with_setter(|s: &mut Self, _, new_val| {
                s.height = new_val;
                s.recalculate_dim()
            })
            .with_default(0)
            .done();
    }

    #[method]
    fn can_flow_from_to(
        &self,
        #[base] _owner: TRef<'_, Resource>,
        from: Vector2,
        to: Vector2,
    ) -> bool {
        match round_vec(to).and_then(|round1| Ok((round1, round_vec(from)?))) {
            Err(msg) => {
                godot_error!("BakedFlowFields: {}", msg);
                false
            }
            Ok(((to_x, to_y), (from_x, from_y))) => {
                self.dim.in_bounds(to_x, to_y)
                    && unsafe {
                        self.flow_fields[self.dim.project_to_field_idx(to_x, to_y)].assume_safe()
                    }
                    .map(|ff, _| ff.can_flow_internal((from_x, from_y)))
                    .unwrap_or_else(|e| {
                        godot_error!("BakedFlowFields: Error querying baked flow field {}", e);
                        false
                    })
            }
        }
    }

    #[method]
    fn flow_from_to(
        &self,
        #[base] _owner: TRef<'_, Resource>,
        from: Vector2,
        to: Vector2,
    ) -> Vector2 {
        match round_vec(to).and_then(|round1| Ok((round1, round_vec(from)?))) {
            Err(msg) => {
                godot_error!("FlowField: {}", msg);
                Vector2::ZERO
            }
            Ok(((to_x, to_y), (from_x, from_y))) => {
                unsafe { self.flow_fields[self.dim.project_to_field_idx(to_x, to_y)].assume_safe() }
                    .map(|ff, _| {
                        ff.flow_internal((from_x, from_y))
                            .map(|(vx, vy)| Vector2 { x: vx, y: vy })
                            .unwrap_or_else(|e| {
                                godot_error!(
                                    "BakedFlowField: Error querying baked flow field {}",
                                    e
                                );
                                Vector2::ZERO
                            })
                    })
                    .unwrap_or_else(|e| {
                        godot_error!("BakedFlowField: Error querying baked flow field {}", e);
                        Vector2::ZERO
                    })
            }
        }
    }
}
