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
    pub fn create(dim: algo::Dimensions, opt_field: Option<algo::FlowField>) -> FlowField {
        FlowField {
            dim,
            width: dim.width() as u64,
            height: dim.height() as u64,
            opt_field,
        }
    }
}

#[derive(NativeClass, ToVariant, FromVariant, Clone)]
#[inherit(Resource)]
#[register_with(Self::register_properties)]
pub struct FlowField {
    dim: algo::Dimensions,
    #[property]
    width: u64,
    #[property]
    height: u64,
    #[property]
    opt_field: Option<algo::FlowField>,
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
            && self.opt_field.as_ref().map_or(false, |field| {
                field[self.dim.project_to_field_idx(from_x, from_y)].is_some()
            })
    }

    fn flow_internal(
        &self,
        from @ (from_x, from_y): algo::Coord,
    ) -> Result<algo::Vector2D, String> {
        if !self.dim.in_bounds(from_x, from_y) {
            return Err(format!("FlowField: position {:#?} out of bounds!", from));
        } else if let Some(field) = &self.opt_field {
            if let Some((vx, vy)) = field[self.dim.project_to_field_idx(from_x, from_y)] {
                return Ok((vx, vy));
            }
        }
        Err(format!(
            "FlowField: unreachable position {:#?} queried!",
            from
        ))
    }
}

#[methods]
impl FlowField {
    fn new(_owner: &Resource) -> Self {
        Self {
            dim: algo::Dimensions::new(0, 0),
            width: 0,
            height: 0,
            opt_field: None,
        }
    }

    fn register_properties(builder: &ClassBuilder<FlowField>) {
        builder
            .property("field")
            .with_getter(|s, _| s.opt_field.to_owned())
            .with_setter(|s: &mut Self, _, new_val: Option<algo::FlowField>| s.opt_field = new_val)
            .with_default(None)
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
            Ok((x, y)) => self.can_flow_internal((x, y)),
        }
    }

    #[method]
    fn flow(&self, #[base] _owner: TRef<'_, Resource>, from: Vector2) -> Vector2 {
        match round_vec(from) {
            Err(msg) => {
                godot_error!("FlowField: {}", msg);
            }
            Ok((x, y)) => match self.flow_internal((x, y)) {
                Err(m) => godot_warn!("FlowField: {}", m),
                Ok((vx, vy)) => return Vector2 { x: vx, y: vy },
            },
        }
        Vector2::ZERO
    }
}

pub struct BakedFlowFieldsFactory {}
impl BakedFlowFieldsFactory {
    pub fn create(dim: Dimensions, fields: Vec<FlowField>) -> BakedFlowFields {
        BakedFlowFields {
            dim,
            width: dim.width() as u64,
            height: dim.height() as u64,
            flow_fields: fields,
        }
    }
}

#[derive(NativeClass, ToVariant, FromVariant)]
#[inherit(Resource)]
#[register_with(Self::register_properties)]
pub struct BakedFlowFields {
    dim: Dimensions,
    #[property]
    width: u64,
    #[property]
    height: u64,
    #[property]
    flow_fields: Vec<FlowField>,
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
            .with_getter(|s, _| s.flow_fields.clone())
            .with_setter(|s: &mut Self, _, new_val: Vec<FlowField>| s.flow_fields = new_val)
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
                    && self.flow_fields[self.dim.project_to_field_idx(to_x, to_y)]
                        .can_flow_internal((from_x, from_y))
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
            Ok(((to_x, to_y), (from_x, from_y))) => self.flow_fields
                [self.dim.project_to_field_idx(to_x, to_y)]
            .flow_internal((from_x, from_y))
            .map(|(vx, vy)| Vector2 { x: vx, y: vy })
            .unwrap_or_else(|e| {
                godot_warn!("BakedFlowField: Error querying baked flow field {}", e);
                Vector2::ZERO
            }),
        }
    }
}
