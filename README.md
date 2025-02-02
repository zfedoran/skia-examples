# Plugy-Examples

This repository contains some examples of how to use skia to draw some text and
shapes on a canvas using rust, harfbuzz and freetype.

I had a hard time finding examples (that just work), so I decided to create
this repository to help others who are looking for examples of how to use skia
with rust.

## Examples

1. Simple [Hello, World](./examples-1/src/main.rs)
2. Emoji [Hello, World 🌎](./examples-2/src/main.rs)
3. Right-to-Left [(Arabic)](./examples-3/src/main.rs)

## Dependencies

These examples were tested on Apple M1, but should work on other platforms.

## Contributing

I'd love to see more examples of how to use skia with rust, so feel free to 
contribute. I'd also appreciate any feedback on how to improve these examples.

In particular, I'd like to see examples of how to use skia with other
languages, and perhaps some with tiny-skia.

## How to build

```bash
cargo build
```

## How to run

```bash
cargo run
```

