## ADDED Requirements

### Requirement: Presence API client and types
The system SHALL provide `frontend/lib/api/presence.ts` with functions `listPersons`, `createPerson`, `deletePerson`, `updateSource`. TypeScript types SHALL include `PresenceState` (`"home" | "away" | "unknown"`), `SourceState` (`"home" | "away" | "unknown"`), `Person` (id, name, grace_period_secs, effective_state, sources), `CreatePersonRequest`, `UpdateSourceRequest`.

#### Scenario: API client exports match backend endpoints
- **WHEN** `listPersons()` is called
- **THEN** it SHALL fetch `GET /api/presence/persons` and return `Person[]`

### Requirement: usePresence SWR hook
The system SHALL provide `frontend/lib/hooks/use-presence.ts` exporting `usePresence()` with `{ persons, isLoading, add, remove, updateSource }`. Mutations SHALL call `mutate()` to revalidate.

#### Scenario: updateSource optimistically reflected
- **WHEN** `updateSource(id, source, state)` is called
- **THEN** the SWR cache SHALL be revalidated after the API call completes

### Requirement: Presence page
The system SHALL provide `frontend/app/presence/page.tsx` displaying all tracked persons. Each card SHALL show: person name, `effective_state` badge (green = home, grey = away, amber = unknown), source breakdown table (source name → state chip), and a manual-override button that calls `updateSource(id, "manual", "home"|"away")`.

#### Scenario: Empty state
- **WHEN** no persons exist
- **THEN** the page SHALL display a prompt to add the first person

#### Scenario: Manual override sets source
- **WHEN** user clicks "Set Home" override button for a person
- **THEN** `updateSource` SHALL be called with source `"manual"` and state `"home"`

### Requirement: Add Person modal
The system SHALL provide `frontend/components/presence/add-person-modal.tsx` with fields: Name (required), Grace Period (seconds, default 120). On submit it SHALL call `add(req)` and close.

#### Scenario: Duplicate name shows error
- **WHEN** the API returns `409 Conflict`
- **THEN** the modal SHALL display an error message without closing

### Requirement: Presence nav link
The system SHALL add `/presence` (label "Presence") to the navigation links in `frontend/components/layout/nav.tsx`.

#### Scenario: Nav link active highlight
- **WHEN** the user navigates to `/presence`
- **THEN** the "Presence" nav link SHALL have the active style applied
