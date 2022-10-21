use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct FlowFieldTileCost {
    #[property(default = false)]
    pub impassable: bool,
    #[property(default = 1.0)]
    pub cost: f32,
}

#[methods]
impl FlowFieldTileCost {
    pub fn new(_owner: &Node) -> Self {
        Self {
            impassable: false,
            cost: 1.0,
        }
    }
}
