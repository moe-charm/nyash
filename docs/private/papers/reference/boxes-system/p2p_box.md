# P2PBox - Modern P2P Node (InProcess)

Status: Experimental (Phase 9.79a)

## Overview
- Structured messaging with `IntentBox(name, payload)`
- Local in-process routing via `MessageBus`
- Deterministic smoke demo available without timers

## API
- `new P2PBox(nodeId: String, transport: String)`
- `send(to: String|Box, intent: IntentBox) -> ResultBox`
- `on(name: String, handler: MethodRef) -> ResultBox`
- `onOnce(name: String, handler: MethodRef) -> ResultBox`
- `off(name: String) -> ResultBox`
- `getNodeId() -> String`
- `isReachable(nodeId: String) -> Bool`
- `getTransportType() -> String`
- `debugNodes() -> String` (inprocess only)
- `debugBusId() -> String` (inprocess only)
- `getLastFrom() -> String` (loopback trace)
- `getLastIntentName() -> String` (loopback trace)

Notes:
- Handlers currently accept a method reference (`MethodBox`) rather than an inline function literal.
- For quick loopback smoke without handlers, send to self and read `getLast*()`.

## Quick Smoke (No Handlers)
```
alice = new P2PBox("alice", "inprocess")
msg = new IntentBox("ping", { })
res = alice.send("alice", msg)
print("last.from=" + alice.getLastFrom())
print("last.intent=" + alice.getLastIntentName())
```

## Two-Node Ping-Pong (Concept)
This is covered in unit tests; handler wiring uses `MethodBox` internally. A higher-level sugar for method references will arrive in later phases.

