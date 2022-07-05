# Move TS

Code generator for Move packages.

_Documentation is currently extremely sparse but will be improved in the near future._

## Setup

Install the CLI using Cargo:

```bash
cargo install move-tsgen

# On Sui
cargo install move-tsgen --features address20

# On Aptos
cargo install move-tsgen --features address32
```

## Usage

In a directory containing a `Move.toml`, run:

```
move-tsgen
```

This will generate a set of TypeScript files in your `build/ts/` directory.

## License

Move TS is licensed under the Apache License, Version 2.0.
