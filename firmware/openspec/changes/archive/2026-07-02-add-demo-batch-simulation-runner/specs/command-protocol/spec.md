## MODIFIED Requirements

### Requirement: Simulation helpers shall remain outside the main command protocol
Simulation mutation helpers MAY exist for tests or demos, but they SHALL NOT be part of the primary `/commands/*` and `/status` HTTP contract. The firmware MAY expose demo/testing-only helper endpoints under `/simulation/*`, but those endpoints SHALL remain explicitly documented as local simulator helpers and SHALL NOT replace the documented external command API.

#### Scenario: Production command surface remains limited
- **WHEN** a client consumes the main firmware HTTP API
- **THEN** only the documented `/commands/*` and `/status` endpoints SHALL be required for the supported command protocol

#### Scenario: Demo-only simulation helpers stay separate
- **WHEN** a local demo or test client uses `/simulation/*`
- **THEN** those endpoints SHALL be documented as local simulator helpers and SHALL NOT change the contract or semantics of the primary command endpoints

## ADDED Requirements

### Requirement: Demo-only simulation persistence endpoints shall support batch artifacts safely
The firmware SHALL expose `POST /simulation/batch-summary` for local simulator batch persistence. `POST /simulation/log-dump` SHALL support batch-mode logs under `demo-logs/<batch_id>/<session_id>/log.json` while remaining backward-compatible with existing non-batch manual log dumps. These endpoints SHALL validate request payloads, sanitize `batch_id` and `session_id`, reject path traversal, create required directories safely, write JSON only inside the demo log area, and return the saved file path.

#### Scenario: Batch summary is persisted under a batch folder
- **WHEN** a local demo client sends `POST /simulation/batch-summary` with a valid batch summary payload and a safe `batch_id`
- **THEN** the firmware SHALL write pretty JSON under `demo-logs/<batch_id>/batch-summary.json` and return a success response containing that saved path

#### Scenario: Batch summary request and response use the exact JSON shape
- **WHEN** a local demo client sends `POST /simulation/batch-summary`
- **THEN** the request body SHALL use the shape `{ "batch_id": "batch-20260628-001", "summary": { "batch_id": "batch-20260628-001", "created_at": "ISO-8601 string", "finished_at": "ISO-8601 string", "requested_runs": 10, "completed_runs": 10, "cancelled": false, "config": {}, "runs": [], "aggregate_metrics": {}, "worst_runs": {} } }`
- **THEN** top-level `batch_id` SHALL be required, `summary` SHALL be required, and `summary.batch_id` SHALL match top-level `batch_id`
- **THEN** the backend SHALL persist exactly the `summary` object as pretty JSON under `demo-logs/<batch_id>/batch-summary.json`
- **THEN** the response body SHALL use the shape `{ "saved_path": "demo-logs/<batch_id>/batch-summary.json" }`

#### Scenario: Invalid batch identifier is rejected
- **WHEN** a local demo client sends a batch summary or log dump request whose `batch_id` or `session_id` attempts path traversal or uses rejected unsafe path characters
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT write any file outside the allowed demo log directory

#### Scenario: Mismatched batch identifiers are rejected
- **WHEN** a local demo client sends `POST /simulation/batch-summary` with a missing `batch_id`, missing `summary`, or a `summary.batch_id` that does not match top-level `batch_id`
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT persist any file

#### Scenario: Batch log dump uses the batch folder layout
- **WHEN** a local demo client sends `POST /simulation/log-dump` with valid batch metadata for `batch_id` and `session_id`
- **THEN** the firmware SHALL persist the log under `demo-logs/<batch_id>/<session_id>/log.json`

#### Scenario: Batch log dump request and response use the exact JSON shape
- **WHEN** a local demo client sends a batch-mode `POST /simulation/log-dump`
- **THEN** the request body SHALL use the shape `{ "batch_id": "batch-20260628-001", "session_id": "demo-map-20260628-001-run-001", "log": {} }`
- **THEN** `batch_id`, `session_id`, and `log` SHALL be required for batch-mode log dumps
- **THEN** `log.batch_id`, `log.session_id`, `log.run_index`, `log.batch_total_runs`, and `log.batch_mode = true` SHALL be present in the embedded log object
- **THEN** if `log.batch_id` exists it SHALL match top-level `batch_id`, and if `log.session_id` exists it SHALL match top-level `session_id`
- **THEN** the backend SHALL persist exactly the `log` object under `demo-logs/<batch_id>/<session_id>/log.json`
- **THEN** the response body SHALL use the shape `{ "saved_path": "demo-logs/<batch_id>/<session_id>/log.json" }`

#### Scenario: Mismatched batch log identifiers are rejected
- **WHEN** a local demo client sends a batch-mode `POST /simulation/log-dump` with missing `batch_id`, missing `session_id`, missing `log`, or mismatched top-level and embedded `batch_id` or `session_id`
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT persist any file

#### Scenario: Manual log dump remains backward-compatible
- **WHEN** a local demo client sends `POST /simulation/log-dump` without batch metadata using the existing manual log format
- **THEN** the firmware SHALL continue to persist the log successfully using the documented non-batch session layout
