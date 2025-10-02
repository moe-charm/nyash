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
            (Some("TimerBox"), "now_ms", 0) => Some(CallRoute::DirectExtern {
                iface: "nyrt.time",
                method: "now_ms",
            }),
            _ => None,
        };
        if self.trace {
            match route {
                Some(r) => eprintln!("[CallRouting] route={:?} recv={:?} method={} argc={}", r, receiver_origin, method, arg_count),
                None => eprintln!("[CallRouting] fallback recv={:?} method={} argc={}", receiver_origin, method, arg_count),
            }
        }
        route
    }
}
