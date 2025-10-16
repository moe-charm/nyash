//! tables.rs — Declarative host routing tables for MethodRouterBox
//!
//! Responsibility
//! - Describe HostHandle slot delegations in a data structure.
//! - Keep environment gates central so callers do not hand-roll lookups.
//! - Provide small helpers to evaluate whether a route is active.

/// Boolean environment toggle (primary + alias keys)
#[derive(Clone, Copy)]
pub struct EnvToggle {
    pub keys: &'static [&'static str],
}

impl EnvToggle {
    pub const fn new(keys: &'static [&'static str]) -> Self {
        Self { keys }
    }

    #[inline]
    pub fn enabled(self) -> bool {
        crate::runtime::env_gate_box::bool_any(self.keys)
    }
}

/// HostHandle slot route description.
#[derive(Clone, Copy)]
pub struct HostSlotRoute {
    pub names: &'static [&'static str],
    pub arity: usize,
    pub slot: u64,
    pub returns_value: bool,
    pub gate: EnvToggle,
}

impl HostSlotRoute {
    pub const fn new(
        names: &'static [&'static str],
        arity: usize,
        slot: u64,
        returns_value: bool,
        gate: EnvToggle,
    ) -> Self {
        Self {
            names,
            arity,
            slot,
            returns_value,
            gate,
        }
    }

    #[inline]
    pub fn matches(self, method: &str, arity: usize) -> bool {
        self.arity == arity && self.names.iter().any(|name| *name == method)
    }

    #[inline]
    pub fn enabled(self, group_on: bool) -> bool {
        if group_on {
            true
        } else {
            self.gate.enabled()
        }
    }
}

/// Collection of routes that optionally share a group toggle.
#[derive(Clone, Copy)]
pub struct HostSlotRoutes {
    pub routes: &'static [HostSlotRoute],
    pub group_gate: Option<EnvToggle>,
}

impl HostSlotRoutes {
    pub const fn new(routes: &'static [HostSlotRoute], group_gate: Option<EnvToggle>) -> Self {
        Self { routes, group_gate }
    }

    #[inline]
    fn group_enabled(self) -> bool {
        self.group_gate.map(|gate| gate.enabled()).unwrap_or(false)
    }

    #[inline]
    pub fn pick(self, method: &str, arity: usize) -> Option<HostSlotRoute> {
        let group_on = self.group_enabled();
        self.routes
            .iter()
            .copied()
            .find(|route| route.matches(method, arity) && route.enabled(group_on))
    }
}

// --- Tables -----------------------------------------------------------------

use crate::runtime::host_handle_router::consts as slots;

const STRING_NAMES: &[&str] = &["size", "len", "length"];
const ARRAY_NAMES: &[&str] = &["size", "len", "length"];

const MAP_FORCE_ALL: EnvToggle = EnvToggle::new(&["HAKO_MAP_FORCE_HOST", "NYASH_MAP_FORCE_HOST"]);

const ARRAY_FORCE_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_ARRAY_FORCE_HOST", "NYASH_ARRAY_FORCE_HOST"]);
const MAP_SIZE_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_MAP_SIZE_FORCE_HOST", "NYASH_MAP_SIZE_FORCE_HOST"]);
const MAP_HAS_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_MAP_HAS_FORCE_HOST", "NYASH_MAP_HAS_FORCE_HOST"]);
const MAP_GET_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_MAP_GET_FORCE_HOST", "NYASH_MAP_GET_FORCE_HOST"]);
const MAP_SET_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_MAP_SET_FORCE_HOST", "NYASH_MAP_SET_FORCE_HOST"]);
const MAP_KEYS_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_MAP_KEYS_FORCE_HOST", "NYASH_MAP_KEYS_FORCE_HOST"]);
const MAP_VALUES_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_MAP_VALUES_FORCE_HOST", "NYASH_MAP_VALUES_FORCE_HOST"]);

const STRING_LEN_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_STRING_SIZE_FORCE_HOST", "NYASH_STRING_SIZE_FORCE_HOST"]);
const ARRAY_LEN_TOGGLE: EnvToggle =
    EnvToggle::new(&["HAKO_ARRAY_SIZE_FORCE_HOST", "NYASH_ARRAY_SIZE_FORCE_HOST"]);

pub const STRING_HOST_ROUTES: HostSlotRoutes = HostSlotRoutes::new(
    &[HostSlotRoute::new(STRING_NAMES, 0, slots::STRING_LEN, true, STRING_LEN_TOGGLE)],
    None,
);

pub const ARRAY_HOST_ROUTES: HostSlotRoutes = HostSlotRoutes::new(
    &[
        HostSlotRoute::new(ARRAY_NAMES, 0, slots::ARRAY_SIZE, true, ARRAY_LEN_TOGGLE),
        HostSlotRoute::new(&["get"], 1, slots::ARRAY_GET, true, ARRAY_FORCE_TOGGLE),
        HostSlotRoute::new(&["set"], 2, slots::ARRAY_SET, false, ARRAY_FORCE_TOGGLE),
    ],
    Some(ARRAY_FORCE_TOGGLE),
);

pub const MAP_HOST_ROUTES: HostSlotRoutes = HostSlotRoutes::new(
    &[
        HostSlotRoute::new(&["size"], 0, slots::MAP_SIZE, true, MAP_SIZE_TOGGLE),
        HostSlotRoute::new(&["has"], 1, slots::MAP_HAS, true, MAP_HAS_TOGGLE),
        HostSlotRoute::new(&["get"], 1, slots::MAP_GET, true, MAP_GET_TOGGLE),
        HostSlotRoute::new(&["set"], 2, slots::MAP_SET, false, MAP_SET_TOGGLE),
        HostSlotRoute::new(&["keys"], 0, slots::MAP_KEYS, true, MAP_KEYS_TOGGLE),
        HostSlotRoute::new(&["values"], 0, slots::MAP_VALUES, true, MAP_VALUES_TOGGLE),
    ],
    Some(MAP_FORCE_ALL),
);
