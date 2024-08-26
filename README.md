# M2RS

M2RS is an experimental projet which targets to entierly rewrite the game [Metin2](https://fr.wikipedia.org/wiki/Metin2) (an old Korean MMORPG) using pure Rust and [wgpu](https://github.com/gfx-rs/wgpu).

## Why I'm doing this

- I love Metin2, Rust, Web, graphics programming and performance critical things
- I was looking for a challenging project
- Because I can and you've probably already heard « L'impossible n'est pas Français 🇫🇷 »

## Features
- `WASM` support
  - [x] std::time replaced by [web_time](https://docs.rs/web-time/latest/web_time/) 
  - [x] assets loading using [reqwest](https://docs.rs/reqwest/latest/reqwest/) 
- `GLTF` support
  - [x] material
  - [x] mesh
  - [x] skeleton
  - [x] animation clip
- `AnimationMixer`
  - [x] frames interpolation
  - [x] blend two clips when playing a new clip
- `Character`
  - [x] allow characters to control it's own animation mixer (pc/npc basically have more than one "wait" animation, we must play them randomly)
  - [ ] dynamic `Object3DInstance` loading using (main) character position
  - [ ] create character controller
  - [ ] basic collisions (using 2d algorithms for performance reasons)
- `ThirdPersonCamera`
  - [ ] third person camera
- `BoneAttachement`
  - [ ] allow `Object3D` to be attached to a skeleton bone (hairs, weapons)
- `Terrain`
  - [ ] parse and generate terrain chunks
  - [ ] shader
  - [ ] shadows
  - [ ] raycast to make characters walk above the ground
  - [ ] objects
  - [ ] trees
  - [ ] water
- `Environment`
  - [ ] sun light that follow a realistic path between day and night 
  - [ ] environment colors
  - [ ] fog
  - [ ] clouds
  - [ ] skybox

## Optimization track
- `GLTF` loader currently produces 4 skeletons if there is 4 skinned mesh linked to the same skeleton.

## Start

To start the game, run the following command and Vulkan or Metal will be used for rendering depending on the platform (Windows, Linux or macOS).

```bash
cargo run --release
```

## Export in the browser (WASM)

In order to export both WASM and JS glue code, you will need to install [wasm-pack](https://github.com/rustwasm/wasm-pack).

```bash
cargo install wasm-pack
```

Then, you can use `wasm-pack` command to export the game.

```bash
wasm-pack build --target web
```

Finally, use a live server to serve the content and open index.html on the selected port. I personnally use [Live Server VSCode extension](https://marketplace.visualstudio.com/items?itemName=ritwickdey.LiveServer).