## ADDED Requirements

### Requirement: Script creation
The system SHALL allow users to create named scripts with an optional description, a list of input parameter definitions, and an ordered list of steps. A script name SHALL be unique (case-insensitive). The system SHALL reject creation if a script with the same name already exists.

#### Scenario: Successful script creation
- **WHEN** a `POST /api/scripts` request is received with a valid name and step list
- **THEN** the system stores the script, assigns a UUID, persists it to the database, and returns `201` with the full script object

#### Scenario: Duplicate name rejected
- **WHEN** a `POST /api/scripts` request is received with a name that matches an existing script (case-insensitive)
- **THEN** the system returns `409 Conflict`

### Requirement: Script step types
Each script step SHALL be one of: `set_state` (device_name, state), `set_brightness` (device_name, brightness 0–100), `set_temperature` (device_name, mireds), `delay` (milliseconds, max 60000), `apply_scene` (scene_name), `call_script` (script_name, args). Template expressions SHALL be permitted in device_name, state, brightness, and temperature fields.

#### Scenario: Valid step types accepted
- **WHEN** a script is created with steps of any supported type
- **THEN** all steps are stored and returned verbatim in the script object

#### Scenario: Unknown step type rejected
- **WHEN** a script creation request includes a step with an unrecognized type field
- **THEN** the system returns `400 Bad Request`

### Requirement: Script execution
The system SHALL execute a script via `POST /api/scripts/{id}/run` with an optional `args` object. Execution SHALL run asynchronously (spawned task). The endpoint SHALL return `202 Accepted` immediately. Steps are executed in order; `delay` steps sleep the task. `call_script` steps invoke another script recursively up to depth 5.

#### Scenario: Script runs successfully
- **WHEN** `POST /api/scripts/{id}/run` is called with valid args
- **THEN** the system returns `202` and the script executes its steps against SmartHome state

#### Scenario: Max recursion depth exceeded
- **WHEN** a script calls another script and the call depth reaches 6
- **THEN** execution halts and the device `last_error` for any affected device is set to `"script: max depth exceeded"`

#### Scenario: Unknown script run requested
- **WHEN** `POST /api/scripts/{id}/run` is called with an ID that does not exist
- **THEN** the system returns `404 Not Found`

### Requirement: Script CRUD
The system SHALL support: `GET /api/scripts` (list all), `GET /api/scripts/{id}` (get one), `PUT /api/scripts/{id}` (full replace), `DELETE /api/scripts/{id}` (remove). Deleting a script that is referenced by an automation `script_call` action SHALL succeed but leave the automation referencing a now-missing script (soft-delete behavior).

#### Scenario: List scripts
- **WHEN** `GET /api/scripts` is called
- **THEN** the system returns an array of all scripts ordered by name

#### Scenario: Delete script
- **WHEN** `DELETE /api/scripts/{id}` is called
- **THEN** the system removes the script from memory and DB and returns `204 No Content`
