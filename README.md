# Ternary Cargo

Resource transport and logistics between rooms with **ternary conservation verification**. Models cargo holds, trade routes, manifests, and inspection systems for multi-room agent economies where every resource carries a ternary weight {-1, 0, +1}.

## Why It Matters

In multi-agent systems that span multiple "rooms" (logical partitions, physical zones, or compute nodes), resource flow is the circulatory system. Every resource — data, energy, agents, blueprints, computation — carries a ternary weight that classifies its contribution to the system:

- **+1 (Positive)**: Net value-adding resources (energy, completed computation)
- **0 (Neutral)**: Inert resources (raw data, empty containers)
- **-1 (Negative)**: Depleting resources (debt, decay, contraband)

The fundamental invariant is **conservation of ternary charge**: the total value across all rooms must remain constant during transport. No value is created or destroyed — only moved. The `CargoInspector` enforces this with cryptographic-style manifest verification.

## How It Works

### Ternary Resource Model

Each resource has a type, a ternary weight, and a quantity:

$$V(R) = w \cdot q, \quad w \in \{-1, 0, +1\}, \quad q \in \mathbb{Z}^+$$

The total value of a cargo hold is:

$$V_{\text{hold}} = \sum_{i} V(R_i) = \sum_{i} w_i \cdot q_i$$

### Conservation Law

For transport from room A to room B:

$$V_{\text{before}}(A) + V_{\text{before}}(B) = V_{\text{after}}(A) + V_{\text{after}}(B)$$

The `CargoInspector::verify_conservation(before, after)` function checks this invariant in O(1).

### Transport Cost

Trade routes have a distance $d$ and compute transport cost as:

$$C_{\text{transport}} = \left\lfloor \frac{d \cdot Q_{\text{total}}}{10} \right\rfloor$$

where $Q_{\text{total}}$ is the total quantity being shipped.

### Inspection Pipeline

The full inspection verifies three properties:
1. **Value conservation**: declared value == actual value
2. **No contraband**: no negative-weight resources in the manifest
3. **Manifest match**: declared items match loaded items

### Complexity

| Operation | Time |
|-----------|------|
| `CargoHold::store(resource)` | O(R) — R = distinct resource types |
| `CargoHold::withdraw(kind, weight, qty)` | O(R) |
| `CargoShip::load(manifest)` | O(M) — M = manifest items |
| `CargoShip::travel(route)` | O(1) |
| `CargoInspector::full_inspect(...)` | O(M) |
| `Smuggler::clean_manifest(items)` | O(H · I) — H = hidden, I = items |

## Quick Start

```rust
use ternary_cargo::{
    CargoHold, CargoShip, TradeRoute, Manifest, CargoInspector,
    TernaryResource, ResourceKind, TernaryWeight,
};

// Create resources
let energy = TernaryResource::new(ResourceKind::Energy, TernaryWeight::Positive, 50);
let debt   = TernaryResource::new(ResourceKind::Data, TernaryWeight::Negative, 10);

// Store in a cargo hold
let mut hold = CargoHold::new("room-a", 100);
assert!(hold.store(energy));
assert_eq!(hold.total_ternary_value(), 50);

// Create a manifest and ship
let manifest = Manifest::new("m1", "room-a", "room-b".into(), vec![energy]);
let mut ship = CargoShip::new("s1", "room-a", 100);
assert!(ship.load(&manifest));

let route = TradeRoute::new("r1", "room-a", "room-b", 10);
assert!(ship.travel(&route).is_ok());

// Inspect
let result = CargoInspector::full_inspect(50, &manifest, &manifest.items);
assert!(result.passed);
```

## API

### Resource Types

| Type | Description |
|------|-------------|
| `TernaryResource` | kind + ternary_weight + quantity |
| `ResourceKind` | Enum: Data, Energy, Agent, Blueprint, Computation |
| `TernaryWeight` | Enum: Positive (+1), Neutral (0), Negative (-1) |

### Transport

| Type | Description |
|------|-------------|
| `CargoHold` | Room-local storage with capacity limit |
| `CargoShip` | Transport vessel with load/unload/travel |
| `TradeRoute` | Origin → destination with distance and active flag |
| `Manifest` | Declared cargo with verified value |

### Inspection

| Function | Description |
|----------|-------------|
| `CargoInspector::verify_conservation(before, after)` | Ternary charge conservation |
| `CargoInspector::verify_no_contraband(manifest)` | No negative-weight items |
| `CargoInspector::verify_manifest(manifest, actual)` | Manifest matches reality |
| `CargoInspector::full_inspect(...)` | All three checks combined |

### Adversarial

| Type | Description |
|------|-------------|
| `Smuggler` | Hides negative-weight resources from manifests |

## Architecture Notes

The cargo system is the physical manifestation of the **γ + η = C** conservation principle:

- **γ (structure)**: the trade route topology — which rooms connect to which, their distances
- **η (dynamics)**: the flow of resources — ships carrying manifests between rooms
- **C (conservation)**: total ternary charge across all rooms is invariant — $\sum_i V_i = \text{const}$

The `Smuggler` type represents the violation of conservation — negative-value resources hidden from inspection. When a smuggler successfully conceals a -1 weighted resource, the declared γ no longer matches the actual γ, breaking the conservation invariant C. The inspector's job is to detect this η-perturbation.

## References

- Samuelson, P. (1947). *Foundations of Economic Analysis*. — Conservation laws in economics.
- Shoham, Y. & Leyton-Brown, K. (2009). *Multiagent Systems*. Cambridge — Resource allocation mechanisms.
- Lamport, L. (1978). *Time, Clocks, and the Ordering of Events in a Distributed System*. CACM — Conservation in distributed state.

## License: MIT
