#![feature(iterator_try_collect)]
use gdnative::prelude::*;

mod algo;
mod flowfield;
mod generator;
mod tilecost;

fn init(handle: InitHandle) {
    handle.add_class::<flowfield::FlowField>();
    handle.add_class::<flowfield::BakedFlowFields>();
    handle.add_tool_class::<tilecost::FlowFieldTileCost>();
    handle.add_tool_class::<generator::FlowFieldGenerator>();
}

godot_init!(init);
