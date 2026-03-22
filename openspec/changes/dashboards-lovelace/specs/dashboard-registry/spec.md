## ADDED Requirements

### Requirement: Dashboard domain model
The system SHALL provide a `Dashboard` struct with `id` (UUID), `name` (non-empty string), `icon` (optional string), `views: Vec<View>`, and `created_at: DateTime<Utc>`.
A `View` SHALL have `id` (UUID), `title` (non-empty string), `icon` (optional string), and `cards: Vec<Card>`.
A `Card` SHALL have `id` (UUID), `title` (optional string override), `content: CardContent` which is a serde-tagged enum discriminated by `card_type`.

#### Scenario: Dashboard is created with unique ID
- **WHEN** `Dashboard::new(name, icon)` is called
- **THEN** a new Dashboard is returned with a UUID id, empty views list, and the provided name/icon

#### Scenario: View is created with unique ID
- **WHEN** `View::new(title, icon)` is called
- **THEN** a new View is returned with a UUID id and empty cards list

#### Scenario: Card wraps typed content
- **WHEN** `Card::new(content)` is called with `CardContent::EntityCard { entity_id }`
- **THEN** a Card is returned with a UUID id, no title override, and the given content

### Requirement: Card types
The system SHALL support the following card types as variants of `CardContent`:
- `EntityCard { entity_id: String }` — displays the entity's current state
- `GaugeCard { entity_id: String, min: f64, max: f64, unit: Option<String> }` — numeric gauge display
- `ButtonCard { entity_id: String, action: String }` — tap to execute (action: "toggle" or "script:<name>")
- `StatCard { title: String, entity_ids: Vec<String>, aggregation: String }` — aggregate metric (count/sum/avg)
- `HistoryCard { entity_id: String, hours: u32 }` — time-series sparkline (stub)

#### Scenario: Entity card serializes with card_type discriminator
- **WHEN** `CardContent::EntityCard { entity_id: "device.lamp.switch".into() }` is serialized to JSON
- **THEN** the JSON contains `"card_type": "entity_card"` and `"entity_id": "device.lamp.switch"`

#### Scenario: Gauge card preserves min/max/unit
- **WHEN** a GaugeCard with min=0, max=100, unit=Some("°C") is serialized and deserialized
- **THEN** all fields round-trip correctly

### Requirement: Dashboard registry
The system SHALL provide a `DashboardRegistry` with dual-index (`by_id: HashMap<String, Dashboard>`) and methods: `add`, `get`, `get_mut`, `remove`, `list`.
Duplicate name SHALL return `DomainError::Conflict`.

#### Scenario: Adding two dashboards with different names succeeds
- **WHEN** two dashboards with distinct names are added to the registry
- **THEN** `list()` returns both dashboards

#### Scenario: Duplicate name returns Conflict error
- **WHEN** a dashboard with an already-registered name is added
- **THEN** `DomainError::Conflict` is returned and the registry is unchanged

#### Scenario: Remove returns the dashboard
- **WHEN** `remove(id)` is called for an existing dashboard
- **THEN** the dashboard is returned and is no longer in `list()`

### Requirement: Default dashboard seeding
The system SHALL seed a default "Home" dashboard with one "Overview" view and no cards when the database-backed startup finds zero dashboards.

#### Scenario: Default dashboard is created when none exist
- **WHEN** the server starts with an empty `dashboards` table
- **THEN** one dashboard named "Home" with one view named "Overview" is present in the registry

#### Scenario: Default dashboard is not duplicated on restart
- **WHEN** the server restarts and the "Home" dashboard already exists in the DB
- **THEN** no additional dashboard is created
