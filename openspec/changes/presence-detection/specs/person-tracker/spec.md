## ADDED Requirements

### Requirement: PersonTracker entity with multi-source aggregation
The system SHALL provide a `PersonTracker` domain struct containing an `id` (UUID), `name` (unique, case-insensitive), `grace_period_secs: u32` (default 120), `sources: HashMap<String, SourceState>`, and `away_since: Option<DateTime<Utc>>`.

`SourceState` SHALL be an enum with variants `Home`, `Away`, and `Unknown`.

#### Scenario: New person has unknown effective state
- **WHEN** a `PersonTracker` is created with no sources set
- **THEN** `effective_state(now)` SHALL return `PresenceState::Unknown`

#### Scenario: Any home source makes person home
- **WHEN** at least one source is `SourceState::Home`
- **THEN** `effective_state(now)` SHALL return `PresenceState::Home` regardless of other sources

#### Scenario: All away sources with grace period not elapsed returns home
- **WHEN** all sources are `SourceState::Away` and `away_since` is set to less than `grace_period_secs` ago
- **THEN** `effective_state(now)` SHALL return `PresenceState::Home` (still within grace)

#### Scenario: All away sources with grace period elapsed returns away
- **WHEN** all sources are `SourceState::Away` and `away_since` is set to more than `grace_period_secs` ago
- **THEN** `effective_state(now)` SHALL return `PresenceState::Away`

### Requirement: PresenceRegistry stores and retrieves PersonTrackers
The system SHALL provide a `PresenceRegistry` with `add`, `get`, `get_by_name`, `remove`, `list`, and `update_source` methods. Names SHALL be keyed case-insensitively. Duplicate names SHALL be rejected with a `DomainError::Conflict`.

#### Scenario: Duplicate name rejected
- **WHEN** two persons with the same name (case-insensitive) are added
- **THEN** the second `add` SHALL return `Err(DomainError::Conflict(…))`

#### Scenario: update_source triggers grace-period reset
- **WHEN** `update_source` is called with `SourceState::Home` on a person whose `away_since` is set
- **THEN** `away_since` SHALL be cleared (`None`)

#### Scenario: update_source sets away_since when all sources go away
- **WHEN** `update_source` is called with `SourceState::Away` and all other sources are also `Away` or `Unknown`, and `away_since` is `None`
- **THEN** `away_since` SHALL be set to `now`
