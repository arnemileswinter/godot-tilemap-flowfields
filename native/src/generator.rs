use gdnative::api::{Node, TileMap};
use gdnative::prelude::*;
use rayon::prelude::*;

use crate::algo;
use crate::flowfield::BakedFlowFieldsFactory;
use crate::tilecost::{self};

#[derive(NativeClass, ToVariant, FromVariant, Default)]
#[register_with(Self::register_properties)]
#[inherit(Node)]
pub struct FlowFieldGenerator {
    pub tile_map_path: NodePath,
}

#[methods]
impl FlowFieldGenerator {
    fn new(_base: &Node) -> Self {
        FlowFieldGenerator::default()
    }

    fn register_properties(builder: &ClassBuilder<FlowFieldGenerator>) {
        builder
            .property::<NodePath>("tile_map_path")
            .with_getter(|n: &Self, _base: TRef<Node>| n.tile_map_path.new_ref())
            .with_setter(|n: &mut Self, _base: TRef<Node>, new_value: NodePath| {
                n.tile_map_path = new_value
            })
            .with_default(NodePath::default())
            .done();
    }

    fn find_tile_cost(&self, base: TRef<'_, Node>, name: &str) -> Result<Option<f32>, GodotString> {
        let n = base.find_node(name, false, true).ok_or_else(|| {
            let err: GodotString =
                format!("Child FlowFieldTileCost for tile {} not found!", name).into();
            err
        })?;
        let c: TInstance<tilecost::FlowFieldTileCost> =
            unsafe { n.assume_safe() }.cast_instance().ok_or_else(|| {
                let err: GodotString = format!("Child {} is not a FlowFieldTileCost.", name).into();
                err
            })?;
        c.map(|a, _| if !a.impassable { Some(a.cost) } else { None })
            .map_err(|e| {
                let err: GodotString = format!("Error borrowing tile cost user data: {}", e).into();
                err
            })
    }

    fn get_tile_map(&self, base: TRef<'_, Node>) -> Result<TRef<'_, TileMap>, GodotString> {
        if self.tile_map_path.is_empty() {
            Err("tile_map_path is not assigned.".into())
        } else if let Some(t) = unsafe { base.get_node_as::<TileMap>(self.tile_map_path.new_ref()) }
        {
            Ok(t)
        } else {
            Err("tile_map_path does not lead to a TileMap!".into())
        }
    }

    #[method]
    fn _get_configuration_warning(&self, #[base] base: TRef<'_, Node>) -> GodotString {
        let mut err = self.get_tile_map(base).err().unwrap_or_default();
        err += self.generate_cost_field(base).err().unwrap_or_default();
        err
    }

    fn get_map_dimensions(&self, base: TRef<'_, Node>) -> Result<algo::Dimensions, GodotString> {
        let tm = self.get_tile_map(base)?;
        let Vector2 { x, y } = tm.get_used_rect().size;
        if x < 1.0 {
            Err(format!("Map has illegal size. Got x: {}", x).into())
        } else if y < 1.0 {
            Err(format!("Map has illegal size. Got y: {}", y).into())
        } else {
            Ok(algo::Dimensions::new(x as usize, y as usize))
        }
    }

    fn generate_cost_field(
        &self,
        base: TRef<'_, Node>,
    ) -> Result<(algo::Dimensions, algo::CostField), GodotString> {
        let tm = self.get_tile_map(base)?;
        let ts_ref = tm.tileset().ok_or_else(|| {
            let err: GodotString = "tilemap has no tileset assigned!".into();
            err
        })?;
        let dim = self.get_map_dimensions(base)?;
        let ts = unsafe { ts_ref.assume_safe() };
        let cost = (0..dim.height())
            .flat_map(|y: usize| {
                (0..dim.width()).map(move |x: usize| match tm.get_cell(x as i64, y as i64) {
                    TileMap::INVALID_CELL => Ok(None),
                    t_idx => self.find_tile_cost(base, &ts.tile_get_name(t_idx).to_string()),
                })
            })
            .try_collect()?;
        Ok((dim, cost))
    }

    /**
     * Calculate a single flow field to position.
     * Prints errors to console.
     */
    #[method]
    fn calculate_flow_field(
        &self,
        #[base] base: TRef<'_, Node>,
        to: Vector2,
    ) -> Option<Instance<crate::flowfield::FlowField>> {
        let Vector2 { x: to_xf, y: to_yf } = to;
        if to_xf.is_nan() || to_xf.is_infinite() || to_yf.is_nan() || to_yf.is_infinite() {
            godot_error!(
                "FlowFieldGenerator: Bad Parameters. Got {}",
                to.to_variant()
            );
            None
        } else {
            let (to_x, to_y) = (to_xf as isize, to_yf as isize);
            match self.generate_cost_field(base) {
                Ok((dim, cost)) => {
                    let integration_field =
                        algo::calculate_integration_field(&dim, (to_x, to_y), &cost);
                    let flow_field = algo::calculate_flow_field(&dim, &integration_field);

                    Some(
                        crate::flowfield::FlowFieldFactory::create(dim, flow_field)
                            .emplace()
                            .into_shared(),
                    )
                }
                Err(m) => {
                    godot_error!("FlowFieldGenerator: Error calculating cost map: {}", m);
                    None
                }
            }
        }
    }

    /**
     * Calculate all possible flow fields.
     * Prints errors to console.
     */
    #[method]
    fn bake_flowfields(
        &self,
        #[base] base: TRef<'_, Node>,
    ) -> Option<Instance<crate::flowfield::BakedFlowFields>> {
        match self.generate_cost_field(base) {
            Ok((d, c)) => {
                let dim = &d;
                let cost = &c;
                godot_print!(
                    "FlowFieldGenerator: Baking {} Flow Fields. This might take a while...",
                    dim.max_idx()
                );
                let flow_fields: Vec<crate::flowfield::FlowField> = (0..dim.height() as isize)
                    .flat_map(|y| (0..dim.width() as isize).map(move |x| (x, y)))
                    .collect::<Vec<(isize, isize)>>()
                    .into_par_iter()
                    .map(move |(x, y)| {
                        let integration_field =
                            algo::calculate_integration_field(dim, (x, y), cost);
                        let field = algo::calculate_flow_field(dim, &integration_field);
                        crate::flowfield::FlowFieldFactory::create(d, field)
                    })
                    .collect();
                godot_print!(
                    "FlowFieldGenerator: Successfully baked {} Flow Fields to Resource!",
                    dim.max_idx()
                );
                Some(BakedFlowFieldsFactory::create(d, flow_fields).emplace().into_shared())
            }
            Err(m) => {
                godot_error!("FlowFieldGenerator: Error calculating cost map: {}", m);
                None
            }
        }
    }
}
