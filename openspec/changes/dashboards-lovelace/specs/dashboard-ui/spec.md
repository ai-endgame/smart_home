## ADDED Requirements

### Requirement: Dashboard Viewer page
The system SHALL provide a `/dashboards` page that lists all dashboards and renders a selected dashboard's views as tabs with live-updating cards.

#### Scenario: Dashboard list shown on first visit
- **WHEN** user navigates to `/dashboards`
- **THEN** a list of dashboard names is displayed; clicking one shows its views

#### Scenario: View tabs are rendered for selected dashboard
- **WHEN** a dashboard with multiple views is selected
- **THEN** one tab per view is shown; clicking a tab renders that view's cards

#### Scenario: Empty dashboard shows placeholder
- **WHEN** a dashboard has no views or a view has no cards
- **THEN** an empty-state message is displayed prompting the user to add cards

### Requirement: Card rendering components
The system SHALL provide React components for each card type that poll entity state via `useEntity(entity_id)` with a 3-second refresh interval.

#### Scenario: EntityCard renders entity state and kind
- **WHEN** an EntityCard for `entity_id = "device.lamp.switch"` is rendered
- **THEN** the card shows the entity's current state value and kind label

#### Scenario: GaugeCard renders a percentage fill bar
- **WHEN** a GaugeCard with min=0, max=100 and entity value=72 is rendered
- **THEN** the gauge fill is proportional to (72-0)/(100-0) = 72%

#### Scenario: ButtonCard triggers action on click
- **WHEN** a ButtonCard with `action = "toggle"` is clicked
- **THEN** a PATCH request is issued to toggle the entity's state

#### Scenario: StatCard shows aggregated count
- **WHEN** a StatCard with `aggregation = "count"` and 3 entity_ids is rendered
- **THEN** the card displays 3 as the count

#### Scenario: HistoryCard shows unavailable placeholder
- **WHEN** a HistoryCard is rendered
- **THEN** the card shows a "History unavailable" placeholder message

### Requirement: Dashboard Builder page
The system SHALL provide a `/dashboards/builder` sub-page where users can create/delete dashboards, add/delete views, and add/delete cards to views.

#### Scenario: Create new dashboard
- **WHEN** user submits the "New Dashboard" form with a name
- **THEN** the dashboard is created via the API and appears in the list

#### Scenario: Add view to dashboard
- **WHEN** user clicks "Add View" and submits a title
- **THEN** a view is added to the selected dashboard

#### Scenario: Add entity card to view
- **WHEN** user picks card type "entity" and enters an entity_id, then submits
- **THEN** the card appears in the view

#### Scenario: Delete card from view
- **WHEN** user clicks the delete button on a card in the builder
- **THEN** the card is removed from the view

### Requirement: Nav link for dashboards
The system SHALL include a "Dashboards" link in the navigation bar pointing to `/dashboards`.

#### Scenario: Nav includes Dashboards link
- **WHEN** the nav is rendered
- **THEN** a link labelled "Dashboards" with href "/dashboards" is present
