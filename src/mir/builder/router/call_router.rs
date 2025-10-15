#[derive(Debug, Clone, Copy)]
pub enum CallRoute {
    DirectExtern { iface: &'static str, method: &'static str },
}

pub struct CallRoutingBox {
    enabled: bool,
    trace: bool,
}

impl CallRoutingBox {
    pub fn new() -> Self {
        let enabled = matches!(
            std::env::var("NYASH_USE_CALL_ROUTER").ok().as_deref(),
            Some("1" | "true" | "on")
        );
        let trace = matches!(
            std::env::var("NYASH_CALL_ROUTER_TRACE").ok().as_deref(),
            Some("1" | "true" | "on")
        );
        Self { enabled, trace }
    }

    pub fn decide_method_route(
        &self,
        receiver_origin: Option<&str>,
        method: &str,
        arg_count: usize,
    ) -> Option<CallRoute> {
        if !self.enabled {
            return None;
        }
        let route = match (receiver_origin, method, arg_count) {
            (Some("TimerBox"), "now_ms", 0) if crate::common::extern_registry::exists("nyrt.time", "now_ms") => Some(CallRoute::DirectExtern {
                iface: "nyrt.time",
                method: "now_ms",
            }),
            (Some("ArrayBox"), "length", 0)
            | (Some("ArrayBox"), "len", 0)
            | (Some("ArrayBox"), "size", 0) if crate::common::extern_registry::exists("nyrt.array", "size") => Some(CallRoute::DirectExtern {
                iface: "nyrt.array",
                method: "size",
            }),
            (Some("MapBox"), "size", 0) if crate::common::extern_registry::exists("nyrt.map", "size") => Some(CallRoute::DirectExtern {
                iface: "nyrt.map",
                method: "size",
            }),
            _ => None,
        };
        if self.trace {
            match route {
                Some(r) => eprintln!(
                    "[CallRouting] enabled route={:?} recv={} method={} argc={}",
                    r,
                    receiver_origin.unwrap_or("<unknown>"),
                    method,
                    arg_count
                ),
                None => eprintln!(
                    "[CallRouting] fallback recv={} method={} argc={}",
                    receiver_origin.unwrap_or("<unknown>"),
                    method,
                    arg_count
                ),
            }
        }
        route
    }
}
