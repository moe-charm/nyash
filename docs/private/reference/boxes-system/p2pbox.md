# P2PBox Reference (Phase 9.79 Minimal)

Status: Experimental (Loopback-ready)
Updated: 2025-08-26

## Overview
P2PBox is a structured P2P node built on IntentBox + MessageBus + Transport.
This document covers the minimal API implemented for Phase 9.79.

## Core API
- new(nodeId: String, transport: String)
- getNodeId() -> String
- getTransportType() -> String
- isReachable(nodeId: String) -> Bool
- send(to: String, intent: IntentBox) -> ResultBox<Bool, Error>
- on(intentName: String, handler: MethodBox) -> ResultBox<Bool, Error>
- getLastIntentName() -> String  // testing helper
- getLastFrom() -> String        // testing helper

Notes:
- send() returns ResultBox; Ok(true) on success, Err(message) on failure
- on() currently wires MethodBox to transport and will call MethodBox.invoke(intent, from)
- MethodBox.invoke is a stub until Interpreter hook is added (see Roadmap)

## Transport
- InProcessTransport only (in-proc MessageBus)
- Transport trait exposes register_intent_handler(intent, cb) used by on()

## Lifecycle
Typical pattern where handler captures 'me':

```nyash
static box Node {
  init { p2p, handle }

  birth() {
    me.p2p = new P2PBox("alice", "inprocess")
    me.handle = new MethodBox(me, "onPing")
    me.p2p.on("ping", me.handle)
  }

  onPing(intent, from) {
    // TODO: requires MethodBox.invoke integration with interpreter
    // print("got " + intent.getName() + " from " + from)
  }
}
```

Destruction order (no strong-cycle):
- Node (me) drops → fields drop (p2p, handle)
- P2PBox drops → InProcessTransport Drop → MessageBus unregister
- Registered handlers released → MethodBox released → me released

## Quick Smoke
```bash
cargo build -j32
./target/debug/nyash local_tests/p2p_self_ping.nyash
# Expect:
# last intent: ping
# last from: alice
```

## Roadmap
- MethodBox.invoke → Interpreter hook (global invoker or dedicated API)
- P2PBox.off(intent) and onOnce(intent, handler)
- WebSocket/WebRTC transports implementing register_intent_handler
- Remove testing helpers (getLast*) once full handler path is live

