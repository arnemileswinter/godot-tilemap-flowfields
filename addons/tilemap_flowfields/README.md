# Godot Tilemap Flowfields

RTS-Style optimized path-finding for crowds of agents. Built for the Godot game engine, written with performance in mind in Rust with [godot-rust](https://godot-rust.github.io).

<div align="center">
  <a href="https://github.com/arnemileswinter/godot-tilemap-flowfields">
    <img src=".screenshots/screenshot1.png" alt="Screenshot 1">
  </a>
</div>


## Installation

Download this repository as zip and unpack to your game folder.
Its not yet currently on the Godot asset place.

## Usage

Create a "FlowFieldGenerator" node and assign to it the TileMap you wish to use.
For each tile, a "FlowFieldTileCost" node must be added as a child Node of the Generator. 

**Note that currently only Euclidean Path-Finding is implemented.**

If you require a different approach, feel free to open an issue or contribute! :)

### AdHoc flow field calculation

Use `$FlowFieldGenerator.calculate_flow_field(to : Vector2)` to retrieve a flow field towards the target vector.
Note that this `to` vector must be in tile-space of your tile-map. Transfer coordinate systems with `TileMap.world_to_map` and `TileMap.to_local` accordingly, before invocation.

The return-value supports a function `flow(to: Vector2)` (with `to` also in map-space) to query the calculated flow field from the agent's position.

Open the [Example Scene](https://github.com/arnemileswinter/godot-tilemap-flowfields/tree/main/addons/tilemap_flowfields/examples/adhoc) to see it all in action.

### Baked flow field calculation

With `$FlowFieldGenerator.bake_flow_fields()` you receive an instance of `BakedFlowFields` supporting the Function `flow_from_to(from:Vector2,to:Vector2)`, with all flow-fields cached. Pathfinding is then happening in constant time.

Baking all flow-fields creates huge files, however, and is also not recommended for scenarios where your game map changes dynamically. It is recommended to use `$FlowFieldGenerator.calculate_flow_field(to : Vector2)`.
Only use baked fields if your map is static and fast-paced path-finding is essential.

The [Baked Example Scene](https://github.com/arnemileswinter/godot-tilemap-flowfields/tree/main/addons/tilemap_flowfields/examples/adhoc) is an example on how to save your baked flow-field as a resource.

## Platforms

Currently Linux/X11 x86_64 and windows-x64 is officially compiled.
If you have the resources to build for darwin or other targets, please don't hesitate to open a PR!

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<div align="center">
  <a href="https://github.com/arnemileswinter/godot-tilemap-flowfields">
    <img src=".screenshots/screenshot2.png" alt="Screenshot 2">
  </a>
</div>

## Compiling

A `justfile` is provided: development binaries are built with `just`, releasing is done via `just build-release`.
The Godot editor must be closed prior to building, else it segfaults because the FlowfieldGenerator is a tool-script.

## Known Issues

- Because of [this issue](https://github.com/godot-rust/godot-rust/issues/905) in the Godot-Engine, it is currently not possible to type-hint the "FlowField" or "BakedFlowFields" Resources properly.
The only type hint that you can use within gdscript `Resource`.

- Not a bug but if your Agent wanders onto an impassable tile (either its cost is impassable or there is no tile at their position) it will no longer move. It is on you to prevent Agent's wandering or pushing one another into such locations. Either by using Physics and proper collision shapes, or by an approach such as Boids or the like.
