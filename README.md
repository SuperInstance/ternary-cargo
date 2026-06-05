# ternary-cargo

Resource transport and logistics between rooms with ternary conservation verification.

## Why This Exists

Fleets move resources between rooms — data, energy, agent components, blueprints. But in a ternary system, resources carry a {-1, 0, +1} weight, and conservation matters: the total ternary value entering a room should match what left the origin. Ternary-cargo provides cargo holds, ships, trade routes, manifests, and a smuggling system for adversarial testing.

## Core Concepts

- **Ternary resource**: A resource (Data, Energy, Agent, Blueprint, Computation) with a ternary weight (Positive, Neutral, Negative) and a quantity. Total value = weight × quantity.
- **Cargo hold**: Storage at a room with capacity limits. Merges resources of the same kind and weight.
- **Manifest**: A declaration of what's being transported, including declared total value. Can be verified against actual cargo.
- **Trade route**: An established path between rooms with a distance metric. Provides transport cost estimates.
- **Cargo ship**: Loads from manifests, travels along routes, unloads at destination. Has its own capacity.
- **Cargo inspector**: Verifies conservation (value before = value after), detects contraband (negative-ternary resources), and validates manifests against actual cargo.
- **Smuggler**: Hides negative-ternary resources to produce "clean" manifests. For testing adversarial scenarios.

## Quick Start

```toml
[dependencies]
ternary-cargo = "0.1"
```

```rust
use ternary_cargo::*;

let mut hold = CargoHold::new("warehouse", 1000);
hold.store(TernaryResource::new(ResourceKind::Energy, TernaryWeight::Positive, 50));

let manifest = Manifest::new("m1", "warehouse", "factory".into(), vec![
    TernaryResource::new(ResourceKind::Energy, TernaryWeight::Positive, 20),
]);

let mut ship = CargoShip::new("hauler-1", "warehouse", 100);
ship.load(&manifest);

let route = TradeRoute::new("r1", "warehouse", "factory", 50);
ship.travel(&route).unwrap();
let delivered = ship.unload();
```

## API Overview

| Type | Description |
|------|-------------|
| `TernaryResource` | Resource with kind, ternary weight, and quantity |
| `ResourceKind` | Data, Energy, Agent, Blueprint, Computation |
| `TernaryWeight` | Positive, Neutral, Negative |
| `CargoHold` | Room-level storage with capacity |
| `Manifest` | Cargo declaration with declared value |
| `TradeRoute` | Established path between rooms |
| `CargoShip` | Vehicle that loads, travels, and unloads |
| `CargoInspector` | Verifies conservation and manifest integrity |
| `Smuggler` | Hides negative resources for adversarial testing |

## How It Works

Transport follows a pipeline: hold → manifest → ship → route → unload. The `CargoInspector` sits at checkpoints, verifying three invariants: (1) ternary value is conserved across transport, (2) no contraband (negative-ternary resources), and (3) manifest matches actual cargo. Any violation fails the inspection.

`CargoHold` merges resources of identical kind and weight automatically — storing 10 Energy+Positive twice results in one entry of 20, not two entries of 10. This simplifies accounting.

`Smuggler` is deliberately provided for adversarial testing: it can hide negative-ternary resources, produce clean manifests that pass superficial inspection, and reveal hidden cargo. This lets you test whether your inspection pipeline catches smuggling attempts.

## Known Limitations

- No multi-hop routing — ships travel one route at a time.
- No partial loads or split shipments.
- Capacity is a simple scalar (total quantity), not per-kind.
- No retry or error recovery for failed transports.
- No concept of transport time — travel is instantaneous.
- Smuggler's clean manifest is naive; a sophisticated inspector could catch quantity discrepancies.

## Use Cases

- **Supply chain logistics**: Move materials between factories, verify nothing is lost or substituted in transit.
- **Game inventory systems**: Players trade items between locations, with inspectors catching cheaters who try to duplicate items.
- **Adversarial testing**: Use the Smuggler to generate smuggling attempts, then verify your CargoInspector catches them.

## Ecosystem Context

Part of the SuperInstance ternary fleet. Works with `ternary-navigator` for route planning, `ternary-shipyard` for transporting agent blueprints, and `ternary-observatory` for monitoring cargo flow. `ternary-beacon` can announce trade route availability.

## License

MIT

## See Also
- **ternary-room** — related
- **ternary-harbor** — related
- **ternary-shipyard** — related
- **ternary-inventory** — related
- **ternary-channel** — related

