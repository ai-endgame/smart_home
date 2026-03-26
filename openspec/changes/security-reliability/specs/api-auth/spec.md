## ADDED Requirements

### Requirement: Optional API key authentication
The system SHALL read an `API_KEY` environment variable at startup. When `API_KEY` is set and non-empty, all HTTP requests with a mutating method (POST, PATCH, DELETE, PUT) SHALL require a valid key presented as either `Authorization: Bearer <key>` or `X-API-Key: <key>`. Requests missing or supplying a wrong key SHALL receive 401 with `{"code": "unauthorized", "message": "invalid or missing API key"}`.

When `API_KEY` is not set, the middleware SHALL pass all requests through without authentication checks.

#### Scenario: Write request with correct API key is accepted
- **GIVEN** `API_KEY = "secret123"` is set in the environment
- **WHEN** `POST /api/devices` is called with `Authorization: Bearer secret123`
- **THEN** the request is forwarded to the handler and returns 201

#### Scenario: Write request missing API key is rejected
- **GIVEN** `API_KEY = "secret123"` is set
- **WHEN** `POST /api/devices` is called with no auth header
- **THEN** 401 is returned with `"code": "unauthorized"`

#### Scenario: Write request with wrong API key is rejected
- **GIVEN** `API_KEY = "secret123"` is set
- **WHEN** `POST /api/devices` is called with `X-API-Key: wrongkey`
- **THEN** 401 is returned

#### Scenario: Read request (GET) always passes through
- **GIVEN** `API_KEY = "secret123"` is set
- **WHEN** `GET /api/devices` is called with no auth header
- **THEN** 200 is returned (GET is exempted from auth)

#### Scenario: OPTIONS preflight always passes through
- **GIVEN** `API_KEY = "secret123"` is set
- **WHEN** `OPTIONS /api/devices` is called
- **THEN** 200/204 is returned (CORS preflight must not be blocked)

#### Scenario: API_KEY absent — all requests pass
- **GIVEN** `API_KEY` is not set
- **WHEN** `POST /api/devices` is called with no auth header
- **THEN** the request proceeds normally (no 401)

### Requirement: API key configuration
The system SHALL expose `api_key: Option<String>` in `Config` (read from `API_KEY` env var) and store it in `AppState`. The middleware SHALL read the key from `AppState` so it can be injected in tests without env var changes.
