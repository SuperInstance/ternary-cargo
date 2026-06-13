# Ternary Cargo

**Ternary Cargo** implements resource transport and logistics between fleet rooms — providing cargo holds, trade routes, manifests, cargo ships, and conservation law verification for ternary-weighted resource exchange.

## Why It Matters

Fleet rooms produce and consume resources: computation, energy, data, and agents. When room A has surplus computation capacity and room B needs it, someone must transport the resource. Ternary Cargo models resources with ternary weights {-1 (costly), 0 (neutral), +1 (beneficial)} and verifies that every trade preserves the conservation law: the total ternary value before transport equals the total after. This catches transport losses, theft, and accounting errors.

## How It Works

### Resource Model

```rust
TernaryResource {
    kind: ResourceKind,        // Data, Energy, Agent, Blueprint, Computation
    ternary_value: TernaryWeight, // Positive (+1), Neutral (0), Negative (-1)
    quantity: u32,
}

total_value = ternary_value.value() × quantity  // i64
```

### Cargo Hold

```rust
CargoHold {
    capacity: u64,
    contents: Vec<TernaryResource>,
    used: u64,
}
```

- `add(resource)` → **O(1)** push, checks capacity
- `remove(kind, quantity)` → **O(N)** scan and split
- `total_value()` → **O(N)** sum

### Trade Route

```rust
TradeRoute {
    source: String,       // room ID
    destination: String,  // room ID
    manifest: Manifest,
    ship: CargoShip,
}
```

Routes validate: conservation (sum of resources unchanged during transport), capacity (ship can carry the load), and compatibility (destination accepts the resource types).

### Manifest and Conservation

```rust
Manifest {
    resources: Vec<TernaryResource>,
    total_ternary: i64,  // Σ (value × quantity)
}

verify_conservation(before: &Manifest, after: &Manifest) → bool {
    before.total_ternary == after.total_ternary
}
```

Conservation check: **O(1)** (compare cached totals). Full verification: **O(N)** per manifest.

### Stealth Transport

For adversarial scenarios, `StealthTransport` routes cargo to avoid detection:

```
path = shortest_path(source, destination, avoid=hostile_rooms)
```

Path finding: **O(V + E)** Dijkstra with hostile room exclusion.

## Quick Start

```rust
use ternary_cargo::{TernaryResource, ResourceKind, TernaryWeight, CargoHold};

let mut hold = CargoHold::new(10_000);
hold.add(TernaryResource::new(ResourceKind::Energy, TernaryWeight::Positive, 500));
hold.add(TernaryResource::new(ResourceKind::Data, TernaryWeight::Neutral, 100));

println!("Total value: {}", hold.total_value()); // 500
println!("Used: {}/{}", hold.used(), hold.capacity());
```

## API

| Type | Description |
|------|-------------|
| `TernaryResource` | kind, ternary weight, quantity |
| `ResourceKind` | Data, Energy, Agent, Blueprint, Computation |
| `TernaryWeight` | Positive (+1), Neutral (0), Negative (-1) |
| `CargoHold` | Bounded storage with add/remove/query |
| `Manifest` | Resource list with conservation total |
| `TradeRoute` | Source, destination, manifest, ship |
| `CargoShip` | Transport vehicle with capacity |

## Architecture Notes

Ternary Cargo provides the logistics layer for resource distribution in SuperInstance. In γ + η = C, Positive (+1) resources represent γ (growth — beneficial resources that expand fleet capability), Negative (-1) resources represent η (avoidance — costly resources that consume capacity), and the conservation law ensures γ + η = C holds during every transport. Integrates with `ternary-bus` for transport scheduling and `ternary-captain` for fleet-level resource allocation.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for fleet logistics architecture.

## References

1. Clark, K. L. et al. (2008). "Multi-Agent Resource Allocation." *AAMAS*.
2. Chevaleyre, Y. et al. (2006). "Issues in Multiagent Resource Allocation." *Informatica*, 30, 3–31.
3. Waldspurger, C. A. & Weihl, W. E. (1994). "Lottery Scheduling: Flexible Proportional-Share Resource Management." *OSDI*.

## License

MIT
