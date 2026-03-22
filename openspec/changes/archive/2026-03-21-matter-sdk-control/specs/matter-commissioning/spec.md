## ADDED Requirements

### Requirement: Commission Matter device via setup code
The server SHALL accept a POST request with a Matter setup code (11-digit manual pairing code or QR code payload) and node ID, invoke `chip-tool pairing code` as a subprocess, and return a job ID for polling.

#### Scenario: Successful commission request
- **WHEN** `POST /api/matter/commission` is called with `{"setup_code": "34970112332", "node_id": 1}`
- **THEN** server returns `202 Accepted` with `{"job_id": "<uuid>", "status": "pending"}`

#### Scenario: Commission job status polling
- **WHEN** `GET /api/matter/commission/{job_id}` is called while job is running
- **THEN** server returns `{"job_id": "...", "status": "in_progress", "message": "Commissioning..."}`

#### Scenario: Commission job completed successfully
- **WHEN** `chip-tool pairing code` exits with code 0
- **THEN** job status changes to `"done"`, `device_id` is returned in the response, and the device is added to `SmartHome` with `control_protocol: matter` and `matter_fabric` populated

#### Scenario: Commission job failed
- **WHEN** `chip-tool pairing code` exits with non-zero code or times out after 60s
- **THEN** job status changes to `"failed"` with `"error"` field containing the chip-tool stderr output

#### Scenario: Invalid setup code format
- **WHEN** `POST /api/matter/commission` is called with a setup code that is not 11 digits
- **THEN** server returns `400 Bad Request` with `{"code": "bad_request", "message": "invalid setup code format"}`

### Requirement: List active commission jobs
The server SHALL expose a `GET /api/matter/commission/jobs` endpoint returning all active and recent commission jobs (last 20).

#### Scenario: List jobs when none exist
- **WHEN** `GET /api/matter/commission/jobs` is called with no jobs in progress
- **THEN** server returns `200 OK` with `[]`

#### Scenario: List jobs with active job
- **WHEN** a commission job is in progress
- **THEN** `GET /api/matter/commission/jobs` returns the job with its current status
