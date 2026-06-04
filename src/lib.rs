#![forbid(unsafe_code)]

//! Ternary Cargo — Resource transport and logistics between rooms.
//!
//! Provides cargo holds, cargo ships, trade routes, manifests, cargo inspection
//! for conservation law verification, and stealth transport for adversarial scenarios.

// ── TernaryResource ────────────────────────────────────────────────────

/// A resource with a ternary value component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TernaryResource {
    pub kind: ResourceKind,
    pub ternary_value: TernaryWeight,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Data,
    Energy,
    Agent,
    Blueprint,
    Computation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TernaryWeight {
    Positive = 1,
    Neutral = 0,
    Negative = -1,
}

impl TernaryWeight {
    pub fn value(self) -> i32 {
        self as i32
    }

    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            -1 => Some(TernaryWeight::Negative),
            0 => Some(TernaryWeight::Neutral),
            1 => Some(TernaryWeight::Positive),
            _ => None,
        }
    }
}

impl TernaryResource {
    pub fn new(kind: ResourceKind, weight: TernaryWeight, quantity: u32) -> Self {
        Self { kind, ternary_value: weight, quantity }
    }

    /// Total ternary value: weight × quantity.
    pub fn total_value(&self) -> i64 {
        self.ternary_value.value() as i64 * self.quantity as i64
    }

    pub fn is_positive(&self) -> bool {
        self.ternary_value == TernaryWeight::Positive
    }
}

// ── CargoHold ──────────────────────────────────────────────────────────

/// Stores ternary resources at a location.
#[derive(Debug, Clone)]
pub struct CargoHold {
    room_id: String,
    resources: Vec<TernaryResource>,
    capacity: u32,
}

impl CargoHold {
    pub fn new(room_id: &str, capacity: u32) -> Self {
        Self { room_id: room_id.to_string(), resources: Vec::new(), capacity }
    }

    pub fn room_id(&self) -> &str {
        &self.room_id
    }

    /// Add a resource. Returns false if would exceed capacity.
    pub fn store(&mut self, resource: TernaryResource) -> bool {
        let used: u32 = self.resources.iter().map(|r| r.quantity).sum();
        if used + resource.quantity > self.capacity {
            return false;
        }
        // Merge with existing of same kind and weight
        if let Some(existing) = self.resources.iter_mut()
            .find(|r| r.kind == resource.kind && r.ternary_value == resource.ternary_value)
        {
            existing.quantity += resource.quantity;
        } else {
            self.resources.push(resource);
        }
        true
    }

    /// Remove resources of a given kind and weight.
    pub fn withdraw(&mut self, kind: ResourceKind, weight: TernaryWeight, quantity: u32) -> Option<TernaryResource> {
        if let Some(idx) = self.resources.iter().position(|r| r.kind == kind && r.ternary_value == weight) {
            if self.resources[idx].quantity >= quantity {
                self.resources[idx].quantity -= quantity;
                let resource = TernaryResource::new(kind, weight, quantity);
                if self.resources[idx].quantity == 0 {
                    self.resources.remove(idx);
                }
                return Some(resource);
            }
        }
        None
    }

    pub fn total_quantity(&self) -> u32 {
        self.resources.iter().map(|r| r.quantity).sum()
    }

    pub fn total_ternary_value(&self) -> i64 {
        self.resources.iter().map(|r| r.total_value()).sum()
    }

    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn resources(&self) -> &[TernaryResource] {
        &self.resources
    }
}

// ── Manifest ───────────────────────────────────────────────────────────

/// Declaration of cargo for transport.
#[derive(Debug, Clone)]
pub struct Manifest {
    pub id: String,
    pub origin: String,
    pub destination: String,
    pub items: Vec<TernaryResource>,
    pub declared_value: i64,
}

impl Manifest {
    pub fn new(id: &str, origin: &str, destination: String, items: Vec<TernaryResource>) -> Self {
        let declared_value = items.iter().map(|r| r.total_value()).sum();
        Self {
            id: id.to_string(),
            origin: origin.to_string(),
            destination,
            items,
            declared_value,
        }
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn total_quantity(&self) -> u32 {
        self.items.iter().map(|r| r.quantity).sum()
    }

    /// Verify the manifest's declared value matches actual item values.
    pub fn verify_value(&self) -> bool {
        let actual: i64 = self.items.iter().map(|r| r.total_value()).sum();
        actual == self.declared_value
    }
}

// ── TradeRoute ─────────────────────────────────────────────────────────

/// An established trade route between two rooms.
#[derive(Debug, Clone)]
pub struct TradeRoute {
    pub id: String,
    pub origin: String,
    pub destination: String,
    pub distance: u32,
    pub active: bool,
}

impl TradeRoute {
    pub fn new(id: &str, origin: &str, destination: &str, distance: u32) -> Self {
        Self { id: id.to_string(), origin: origin.to_string(), destination: destination.to_string(), distance, active: true }
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Estimate transport cost: distance × total quantity / 10.
    pub fn transport_cost(&self, manifest: &Manifest) -> u32 {
        self.distance * manifest.total_quantity() / 10
    }
}

// ── CargoShip ──────────────────────────────────────────────────────────

/// Transports cargo between rooms along trade routes.
#[derive(Debug)]
pub struct CargoShip {
    pub id: String,
    pub location: String,
    pub cargo: Vec<TernaryResource>,
    pub capacity: u32,
}

impl CargoShip {
    pub fn new(id: &str, home: &str, capacity: u32) -> Self {
        Self { id: id.to_string(), location: home.to_string(), cargo: Vec::new(), capacity }
    }

    /// Load cargo from a manifest. Returns false if overweight.
    pub fn load(&mut self, manifest: &Manifest) -> bool {
        let total: u32 = self.cargo.iter().map(|r| r.quantity).sum::<u32>()
            + manifest.items.iter().map(|r| r.quantity).sum::<u32>();
        if total > self.capacity {
            return false;
        }
        self.cargo.extend(manifest.items.clone());
        true
    }

    /// Unload all cargo and return it.
    pub fn unload(&mut self) -> Vec<TernaryResource> {
        std::mem::take(&mut self.cargo)
    }

    /// Travel to destination along a trade route.
    pub fn travel(&mut self, route: &TradeRoute) -> Result<(), String> {
        if !route.active {
            return Err("Route is inactive".into());
        }
        if self.location != route.origin {
            return Err(format!("Ship at {} but route starts at {}", self.location, route.origin));
        }
        self.location = route.destination.clone();
        Ok(())
    }

    pub fn cargo_quantity(&self) -> u32 {
        self.cargo.iter().map(|r| r.quantity).sum()
    }

    pub fn cargo_value(&self) -> i64 {
        self.cargo.iter().map(|r| r.total_value()).sum()
    }
}

// ── CargoInspector ─────────────────────────────────────────────────────

/// Verifies conservation laws during transport.
pub struct CargoInspector;

impl CargoInspector {
    /// Verify total ternary value is conserved: before == after.
    pub fn verify_conservation(before: i64, after: i64) -> bool {
        before == after
    }

    /// Verify no negative-ternary resources were introduced (anti-smuggling check).
    pub fn verify_no_contraband(manifest: &Manifest) -> bool {
        manifest.items.iter().all(|r| r.ternary_value != TernaryWeight::Negative)
    }

    /// Verify manifest items match what was actually loaded.
    pub fn verify_manifest(manifest: &Manifest, actual: &[TernaryResource]) -> bool {
        if manifest.items.len() != actual.len() {
            return false;
        }
        for item in &manifest.items {
            let matching: u32 = actual.iter()
                .filter(|r| r.kind == item.kind && r.ternary_value == item.ternary_value)
                .map(|r| r.quantity)
                .sum();
            if matching < item.quantity {
                return false;
            }
        }
        true
    }

    /// Full inspection: conservation + no contraband + manifest match.
    pub fn full_inspect(before_value: i64, manifest: &Manifest, actual: &[TernaryResource]) -> CargoInspectionResult {
        let value_conserved = Self::verify_conservation(before_value, manifest.declared_value);
        let no_contraband = Self::verify_no_contraband(manifest);
        let manifest_match = Self::verify_manifest(manifest, actual);
        CargoInspectionResult {
            value_conserved,
            no_contraband,
            manifest_match,
            passed: value_conserved && no_contraband && manifest_match,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoInspectionResult {
    pub value_conserved: bool,
    pub no_contraband: bool,
    pub manifest_match: bool,
    pub passed: bool,
}

// ── Smuggler ───────────────────────────────────────────────────────────

/// Stealth transport for adversarial scenarios — hides negative-value resources.
#[derive(Debug)]
pub struct Smuggler {
    hidden: Vec<TernaryResource>,
}

impl Smuggler {
    pub fn new() -> Self {
        Self { hidden: Vec::new() }
    }

    /// Hide a negative-ternary resource from inspection.
    pub fn hide(&mut self, resource: TernaryResource) -> bool {
        if resource.ternary_value == TernaryWeight::Negative {
            self.hidden.push(resource);
            true
        } else {
            false
        }
    }

    /// Produce a "clean" manifest that omits hidden items.
    pub fn clean_manifest(&self, all_items: &[TernaryResource]) -> Vec<TernaryResource> {
        all_items.iter()
            .filter(|r| !self.hidden.iter().any(|h| h.kind == r.kind && h.quantity == r.quantity))
            .cloned()
            .collect()
    }

    /// Reveal all hidden resources.
    pub fn reveal(&mut self) -> Vec<TernaryResource> {
        std::mem::take(&mut self.hidden)
    }

    pub fn hidden_count(&self) -> usize {
        self.hidden.len()
    }

    pub fn hidden_value(&self) -> i64 {
        self.hidden.iter().map(|r| r.total_value()).sum()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pos_resource(kind: ResourceKind, qty: u32) -> TernaryResource {
        TernaryResource::new(kind, TernaryWeight::Positive, qty)
    }

    fn neg_resource(kind: ResourceKind, qty: u32) -> TernaryResource {
        TernaryResource::new(kind, TernaryWeight::Negative, qty)
    }

    #[test]
    fn ternary_weight_values() {
        assert_eq!(TernaryWeight::Positive.value(), 1);
        assert_eq!(TernaryWeight::Neutral.value(), 0);
        assert_eq!(TernaryWeight::Negative.value(), -1);
    }

    #[test]
    fn ternary_weight_from_i32() {
        assert_eq!(TernaryWeight::from_i32(1), Some(TernaryWeight::Positive));
        assert_eq!(TernaryWeight::from_i32(0), Some(TernaryWeight::Neutral));
        assert_eq!(TernaryWeight::from_i32(-1), Some(TernaryWeight::Negative));
        assert_eq!(TernaryWeight::from_i32(5), None);
    }

    #[test]
    fn resource_total_value() {
        let r = pos_resource(ResourceKind::Energy, 10);
        assert_eq!(r.total_value(), 10);
        let r = neg_resource(ResourceKind::Data, 5);
        assert_eq!(r.total_value(), -5);
    }

    #[test]
    fn resource_is_positive() {
        assert!(pos_resource(ResourceKind::Energy, 1).is_positive());
        assert!(!neg_resource(ResourceKind::Energy, 1).is_positive());
    }

    #[test]
    fn cargo_hold_store_and_withdraw() {
        let mut hold = CargoHold::new("room-a", 100);
        assert!(hold.store(pos_resource(ResourceKind::Energy, 20)));
        assert_eq!(hold.total_quantity(), 20);
        let withdrawn = hold.withdraw(ResourceKind::Energy, TernaryWeight::Positive, 10);
        assert!(withdrawn.is_some());
        assert_eq!(hold.total_quantity(), 10);
    }

    #[test]
    fn cargo_hold_capacity_limit() {
        let mut hold = CargoHold::new("room-a", 10);
        assert!(hold.store(pos_resource(ResourceKind::Energy, 10)));
        assert!(!hold.store(pos_resource(ResourceKind::Energy, 1)));
    }

    #[test]
    fn cargo_hold_merge_same_kind() {
        let mut hold = CargoHold::new("room-a", 100);
        hold.store(pos_resource(ResourceKind::Energy, 10));
        hold.store(pos_resource(ResourceKind::Energy, 5));
        assert_eq!(hold.resource_count(), 1);
        assert_eq!(hold.total_quantity(), 15);
    }

    #[test]
    fn cargo_hold_ternary_value() {
        let mut hold = CargoHold::new("room-a", 100);
        hold.store(pos_resource(ResourceKind::Energy, 10));
        hold.store(neg_resource(ResourceKind::Data, 5));
        assert_eq!(hold.total_ternary_value(), 10 - 5);
    }

    #[test]
    fn cargo_hold_withdraw_insufficient() {
        let mut hold = CargoHold::new("room-a", 100);
        hold.store(pos_resource(ResourceKind::Energy, 5));
        assert!(hold.withdraw(ResourceKind::Energy, TernaryWeight::Positive, 10).is_none());
    }

    #[test]
    fn manifest_creation() {
        let m = Manifest::new("m1", "a", "b".into(), vec![
            pos_resource(ResourceKind::Energy, 10),
            neg_resource(ResourceKind::Data, 5),
        ]);
        assert_eq!(m.item_count(), 2);
        assert_eq!(m.declared_value, 5);
        assert!(m.verify_value());
    }

    #[test]
    fn manifest_verify_value_tampered() {
        let mut m = Manifest::new("m1", "a", "b".into(), vec![
            pos_resource(ResourceKind::Energy, 10),
        ]);
        m.declared_value = 999; // tamper
        assert!(!m.verify_value());
    }

    #[test]
    fn trade_route_transport_cost() {
        let route = TradeRoute::new("r1", "a", "b", 100);
        let manifest = Manifest::new("m1", "a", "b".into(), vec![
            pos_resource(ResourceKind::Energy, 20),
        ]);
        assert_eq!(route.transport_cost(&manifest), 200);
    }

    #[test]
    fn trade_route_activate_deactivate() {
        let mut route = TradeRoute::new("r1", "a", "b", 10);
        assert!(route.active);
        route.deactivate();
        assert!(!route.active);
        route.activate();
        assert!(route.active);
    }

    #[test]
    fn cargo_ship_load_and_unload() {
        let mut ship = CargoShip::new("s1", "a", 100);
        let manifest = Manifest::new("m1", "a", "b".into(), vec![
            pos_resource(ResourceKind::Energy, 30),
        ]);
        assert!(ship.load(&manifest));
        assert_eq!(ship.cargo_quantity(), 30);
        let cargo = ship.unload();
        assert_eq!(cargo.len(), 1);
        assert_eq!(ship.cargo_quantity(), 0);
    }

    #[test]
    fn cargo_ship_overweight() {
        let mut ship = CargoShip::new("s1", "a", 5);
        let manifest = Manifest::new("m1", "a", "b".into(), vec![
            pos_resource(ResourceKind::Energy, 100),
        ]);
        assert!(!ship.load(&manifest));
    }

    #[test]
    fn cargo_ship_travel_success() {
        let mut ship = CargoShip::new("s1", "a", 50);
        let route = TradeRoute::new("r1", "a", "b", 10);
        assert!(ship.travel(&route).is_ok());
        assert_eq!(ship.location, "b");
    }

    #[test]
    fn cargo_ship_travel_wrong_location() {
        let mut ship = CargoShip::new("s1", "x", 50);
        let route = TradeRoute::new("r1", "a", "b", 10);
        assert!(ship.travel(&route).is_err());
    }

    #[test]
    fn cargo_ship_travel_inactive_route() {
        let mut ship = CargoShip::new("s1", "a", 50);
        let mut route = TradeRoute::new("r1", "a", "b", 10);
        route.deactivate();
        assert!(ship.travel(&route).is_err());
    }

    #[test]
    fn cargo_inspector_conservation() {
        assert!(CargoInspector::verify_conservation(42, 42));
        assert!(!CargoInspector::verify_conservation(42, 41));
    }

    #[test]
    fn cargo_inspector_no_contraband() {
        let clean = Manifest::new("m1", "a", "b".into(), vec![
            pos_resource(ResourceKind::Energy, 10),
        ]);
        assert!(CargoInspector::verify_no_contraband(&clean));

        let dirty = Manifest::new("m2", "a", "b".into(), vec![
            neg_resource(ResourceKind::Data, 5),
        ]);
        assert!(!CargoInspector::verify_no_contraband(&dirty));
    }

    #[test]
    fn cargo_inspector_manifest_match() {
        let items = vec![pos_resource(ResourceKind::Energy, 10)];
        let manifest = Manifest::new("m1", "a", "b".into(), items.clone());
        assert!(CargoInspector::verify_manifest(&manifest, &items));
    }

    #[test]
    fn cargo_inspector_manifest_mismatch() {
        let items = vec![pos_resource(ResourceKind::Energy, 10)];
        let manifest = Manifest::new("m1", "a", "b".into(), vec![
            pos_resource(ResourceKind::Energy, 20),
        ]);
        assert!(!CargoInspector::verify_manifest(&manifest, &items));
    }

    #[test]
    fn cargo_inspector_full_inspect_pass() {
        let items = vec![pos_resource(ResourceKind::Energy, 10)];
        let manifest = Manifest::new("m1", "a", "b".into(), items.clone());
        let result = CargoInspector::full_inspect(10, &manifest, &items);
        assert!(result.passed);
    }

    #[test]
    fn smuggler_hide_negative() {
        let mut sm = Smuggler::new();
        assert!(sm.hide(neg_resource(ResourceKind::Data, 5)));
        assert_eq!(sm.hidden_count(), 1);
        assert_eq!(sm.hidden_value(), -5);
    }

    #[test]
    fn smuggler_refuse_positive() {
        let mut sm = Smuggler::new();
        assert!(!sm.hide(pos_resource(ResourceKind::Energy, 5)));
        assert_eq!(sm.hidden_count(), 0);
    }

    #[test]
    fn smuggler_clean_manifest() {
        let mut sm = Smuggler::new();
        sm.hide(neg_resource(ResourceKind::Data, 5));
        let all = vec![pos_resource(ResourceKind::Energy, 10), neg_resource(ResourceKind::Data, 5)];
        let clean = sm.clean_manifest(&all);
        assert_eq!(clean.len(), 1);
        assert_eq!(clean[0].kind, ResourceKind::Energy);
    }

    #[test]
    fn smuggler_reveal() {
        let mut sm = Smuggler::new();
        sm.hide(neg_resource(ResourceKind::Data, 5));
        let revealed = sm.reveal();
        assert_eq!(revealed.len(), 1);
        assert_eq!(sm.hidden_count(), 0);
    }
}
