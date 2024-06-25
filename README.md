# M2RS

M2RS is an experimental projet which targets to entierly rewrite the game [Metin2](https://fr.wikipedia.org/wiki/Metin2) (an old Korean MMORPG) using pure Rust and [wgpu](https://github.com/gfx-rs/wgpu).

## Goals

- Learn Rust and WebGPU,
- Learn game-focused concepts
- Create a fully fonctional MMORPG in a browser

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

Finally, use a live server to serve the content and open index.html on the select port. I personnally use [Live Server VSCode extension](https://marketplace.visualstudio.com/items?itemName=ritwickdey.LiveServer).