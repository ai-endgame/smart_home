# Smart Home 20-Hour Mastery Plan

## Core Principle
Focus: **protocols + integration + automation**. Everything else is just applying these three.

---

## Session 1 — Foundations & Ecosystem Map (2h)
**Goal:** Understand what actually matters and why most guides waste your time.

**Study (1h 45m)**
- How smart home layers stack: cloud, hub, local, device
- The 4 protocols that matter: **Zigbee, Z-Wave, Matter, Wi-Fi**
- Hub options: Home Assistant vs SmartThings vs Apple Home — why HA wins for serious users
- [Home Assistant Architecture docs](https://developers.home-assistant.io/docs/architecture_index/) — just the overview pages

**15-min Review**
- Draw the stack: cloud ↔ hub ↔ device
- Can you explain why local control beats cloud?
- Which protocol would you choose for a new sensor, and why?

---

## Session 2 — Home Assistant Core (2h)
**Goal:** Spin up HA and understand its data model.

**Study (1h 45m)**
- Install HA OS in a VM (or on a Pi) — [official install guide](https://www.home-assistant.io/installation/)
- Explore: entities, devices, integrations, areas
- `configuration.yaml` basics — what lives here vs UI
- [HA Concepts & Terminology](https://www.home-assistant.io/docs/configuration/concepts/)

**15-min Review**
- What is the difference between a *device* and an *entity*?
- Add one integration (weather or time) from the UI
- Name 3 things stored in `configuration.yaml` vs the UI database

---

## Session 3 — Zigbee Deep Dive (2h)
**Goal:** Understand the dominant low-power mesh protocol.

**Study (1h 45m)**
- Zigbee mesh topology: coordinator, router, end device
- **Zigbee2MQTT** setup — the gold standard for local control
- [Zigbee2MQTT Getting Started](https://www.zigbee2mqtt.io/guide/getting-started/)
- Device pairing, MQTT topics, HA auto-discovery
- Why routers matter (always-on devices extend mesh)

**15-min Review**
- Pair one real or simulated device
- What's the MQTT topic structure for a device named `living_room_sensor`?
- Why is a smart plug valuable beyond just power control?

---

## Session 4 — Matter & Thread (2h)
**Goal:** Understand the future-proof protocol.

**Study (1h 45m)**
- Matter: IP-based, local, multi-ecosystem
- Thread: IPv6 mesh for battery devices (Border Router concept)
- Matter commissioning flow — QR code → fabric
- [Matter specification overview](https://csa-iot.org/developer-resource/matter-introduction/) (the short intro, not the spec)
- Current gaps: no cameras, no complex automations in spec yet

**15-min Review**
- How does Matter differ from Zigbee architecturally?
- What is a Thread Border Router and why do you need one?
- Name 2 real devices that support Matter today

---

## Session 5 — Automations: The Core Value (2h)
**Goal:** Master HA automations — this is where 80% of the value lives.

**Study (1h 45m)**
- Automation anatomy: **trigger → condition → action**
- Trigger types: state, time, sun, numeric_state, webhook
- Conditions: time, state, template
- Actions: service calls, delays, choose, repeat
- [HA Automations docs](https://www.home-assistant.io/docs/automation/)
- Build 3 automations: motion light, sunrise blind, low battery alert

**15-min Review**
- Write the YAML for "turn off all lights at midnight if nobody is home"
- What's the difference between a trigger and a condition?
- When would you use `choose` vs multiple separate automations?

---

## Session 6 — Scripts, Scenes & Templates (2h)
**Goal:** Move from basic automations to reusable, dynamic logic.

**Study (1h 45m)**
- Scripts: reusable action sequences with parameters
- Scenes: device state snapshots
- **Jinja2 templates** in HA — the superpower
  - `{{ states('sensor.temperature') | float }}`
  - Template conditions, dynamic messages
- [HA Templating docs](https://www.home-assistant.io/docs/configuration/templating/)
- Template editor in Developer Tools — use it constantly

**15-min Review**
- Create a script that dims lights to a passed-in brightness level
- Write a template that returns "hot" / "warm" / "cool" based on temperature
- What's the difference between a scene and a script?

---

## Session 7 — Presence Detection (2h)
**Goal:** The hardest problem in smart home — solve it properly.

**Study (1h 45m)**
- Why GPS-only fails (battery, latency)
- **Layered presence**: router ping + BLE + GPS + manual
- Tools: HA Companion App, Unifi integration, bluetooth proxy
- `person` entity vs `device_tracker` entity
- [HA Presence Detection guide](https://www.home-assistant.io/docs/presence-detection/)
- Grace periods and `not_home` delays to avoid false triggers

**15-min Review**
- Design a 3-layer presence system for one person
- What delay would you set before triggering "nobody home"?
- How does the `person` entity aggregate multiple trackers?

---

## Session 8 — Dashboards & Lovelace (2h)
**Goal:** Build a dashboard you'd actually use daily.

**Study (1h 45m)**
- Lovelace architecture: views, cards, entities
- Essential cards: entities, gauge, history-graph, map, button
- **custom-cards via HACS** — mushroom-cards, mini-graph-card
- [HACS install](https://hacs.xyz/docs/use/)
- Layout strategy: room-based vs function-based views
- Mobile vs desktop considerations

**15-min Review**
- Build a single-room view with light controls + sensor readings
- Install one HACS card
- What's the fastest way to add an entity to a dashboard?

---

## Session 9 — Security & Reliability (2h)
**Goal:** Don't build a system that locks you out or gets hacked.

**Study (1h 45m)**
- **Remote access**: Nabu Casa vs Cloudflare Tunnel vs VPN — tradeoffs
- Never expose HA port directly to internet
- Secrets management: `secrets.yaml`, env vars
- Backup strategy: HA built-in + offsite copy
- Redundancy: UPS for hub, wired Ethernet > Wi-Fi for hub
- Lock/alarm automations: always add manual override

**15-min Review**
- Set up a backup schedule
- What's wrong with port-forwarding 8123 to the internet?
- Design a "safe failure" for a smart lock automation

---

## Session 10 — Advanced Patterns & Your Project (2h)
**Goal:** Apply everything; close knowledge gaps specific to this codebase.

**Study (1h 30m)**
- **Node-RED** for visual automation logic (complex flows)
- REST/MQTT APIs for custom integrations — exactly what `smart_home` backend does
- HA REST integration: expose your Rust API as HA entities
- `rest_command` + `rest_sensor` patterns
- Event-driven architecture: webhooks ↔ HA ↔ external apps
- Read through `backend/src/infrastructure/mdns.rs` and `db.rs` with fresh eyes

**30-min Review (extended — final session)**
- Map your Rust backend's `/api/devices` endpoint to an HA `rest_sensor`
- How would you trigger a HA automation from your Rust server via webhook?
- What's the single biggest gap in your current `smart_home` implementation vs production HA?

---

## Quick Reference

| Session | Focus | Key Concept |
|---------|-------|-------------|
| 1 | Ecosystem | Protocols stack |
| 2 | HA Core | Entity model |
| 3 | Zigbee | Mesh + MQTT |
| 4 | Matter | IP-native future |
| 5 | Automations | Trigger→Condition→Action |
| 6 | Templates | Dynamic logic |
| 7 | Presence | Layered detection |
| 8 | Dashboards | Lovelace + HACS |
| 9 | Security | Remote access + backup |
| 10 | Integration | REST/MQTT + your project |

## The 20% that does 80%

1. **Home Assistant** — one hub to rule all protocols
2. **Zigbee2MQTT** — local control, no cloud dependency
3. **Automations with templates** — where all the value is
4. **Layered presence detection** — makes automations actually work
5. **Local-first + backup** — reliability over features
