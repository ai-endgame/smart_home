export type EntityKind =
  | 'switch'
  | 'light'
  | 'sensor'
  | 'binary_sensor'
  | 'cover'
  | 'climate'
  | 'media_player'
  | 'lock'
  | 'camera'
  | 'number'
  | 'select'
  | 'button'
  | 'person';

export interface Entity {
  entity_id: string;
  kind: EntityKind;
  device_id: string;
  name: string;
  state: string;
  unit_of_measurement?: string;
  attributes: Record<string, unknown>;
}

export interface AreaResponse {
  area_id: string;
  name: string;
  floor?: number;
  icon?: string;
  device_count: number;
}

export interface AreaDetailResponse extends AreaResponse {
  devices: Device[];
}

export type Protocol =
  | 'zigbee'
  | 'z_wave'
  | 'matter'
  | 'thread'
  | 'wifi'
  | 'shelly'
  | 'tasmota'
  | 'esphome'
  | 'wled'
  | 'unknown';

export interface ProtocolInfo {
  id: Protocol;
  transport: string;
  local_only: boolean;
  mesh: boolean;
  description: string;
}

export interface ProtocolEntry extends ProtocolInfo {
  device_count: number;
}

export interface EcosystemLayers {
  local_devices: number;
  cloud_devices: number;
}

export interface EcosystemResponse {
  total_devices: number;
  connected_count: number;
  disconnected_count: number;
  unprotocolled_devices: number;
  layers: EcosystemLayers;
  protocols: ProtocolEntry[];
}

export type DeviceType =
  // Lighting
  | 'light'
  // Climate
  | 'thermostat'
  | 'fan'
  // Access control
  | 'lock'
  // Power
  | 'switch'
  | 'outlet'
  // Media & entertainment
  | 'tv'
  | 'speaker'
  | 'media_player'
  // Sensing & safety
  | 'sensor'
  | 'camera'
  | 'alarm'
  // Covers / motorised
  | 'cover'
  // Infrastructure
  | 'hub';
export type DeviceState = 'on' | 'off' | 'unknown';

export type ZigbeeRole = 'coordinator' | 'router' | 'end_device';

export type ThreadRole = 'border_router' | 'router' | 'end_device' | 'sleepy';

export interface MatterFabric {
  fabric_id: string;
  vendor_id: number;
  commissioner: string;
}

export interface MatterStatus {
  devices_seen: number;
  commissioning_count: number;
  last_seen_at: string | null;
}

export interface MatterDeviceResponse {
  id: string;
  name: string;
  host: string;
  vendor_id: number | null;
  product_id: number | null;
  discriminator: number | null;
  commissioning_mode: number;
  thread_role: string | null;
  protocol: string;
}

export interface FabricResponse {
  fabric_id: string;
  vendor_id: number;
  commissioner: string;
  device_count: number;
}

export type CommissionStatus = 'pending' | 'in_progress' | 'done' | 'failed';

export interface CommissionJobResponse {
  job_id: string;
  status: CommissionStatus;
  message: string;
  device_id: string | null;
  error: string | null;
}

export interface MqttStatus {
  connected: boolean;
  broker: string | null;
  topics_received: number;
  last_message_at: string | null;
}

export interface Device {
  id: string;
  name: string;
  device_type: DeviceType;
  state: DeviceState;
  brightness?: number;
  temperature?: number;
  connected: boolean;
  last_error?: string;
  room?: string;
  control_protocol?: Protocol;
  zigbee_role?: ZigbeeRole;
  thread_role?: ThreadRole;
  matter_fabric?: MatterFabric;
  attributes?: Record<string, unknown>;
  power_w?: number;
  energy_kwh?: number;
}

export interface HistoryEntry {
  state: string;
  ts: string;
}

export interface EnergySummaryResponse {
  total_power_w: number;
  devices_reporting: number;
}

export interface CreateDeviceRequest {
  name: string;
  device_type: DeviceType;
}

export interface AddDiscoveredDeviceRequest {
  discovered_id: string;
  device_type: DeviceType;
  name?: string;
}

export interface UpdateDeviceRequest {
  state?: DeviceState;
  brightness?: number;
  temperature?: number;
  connected?: boolean;
}

export interface DiscoveredDevice {
  id: string;
  name: string;
  host: string;
  port: number;
  addresses: string[];
  service_type: string;
  properties: Record<string, string>;
  suggested_type: DeviceType;
  protocol?: string;
}

export type SunEvent = 'sunrise' | 'sunset';
export type NumericAttr = 'brightness' | 'temperature';

export interface TimeRange {
  from: string;
  to: string;
}

export interface Trigger {
  type: 'state_change' | 'temperature_above' | 'temperature_below' | 'time' | 'sun' | 'numeric_state_above' | 'numeric_state_below' | 'webhook';
  device_name?: string;
  target_state?: DeviceState;
  threshold?: number;
  time?: string;
  event?: SunEvent;
  offset_minutes?: number;
  attribute?: NumericAttr;
  id?: string;
}

export interface Action {
  type: 'state' | 'brightness' | 'temperature' | 'notify' | 'script_call';
  device_name?: string;
  state?: DeviceState;
  brightness?: number;
  temperature?: number;
  message?: string;
  script_name?: string;
  args?: Record<string, unknown>;
}

export type ConditionType =
  | 'state_equals'
  | 'brightness_above'
  | 'brightness_below'
  | 'template_eval';

export interface Condition {
  type: ConditionType;
  device_name?: string;
  state?: DeviceState;
  value?: number;
  expr?: string;
}

export interface AutomationRule {
  name: string;
  enabled: boolean;
  safe_mode: boolean;
  trigger: Trigger;
  action: Action;
  time_range?: TimeRange | null;
  conditions?: Condition[];
}

export interface CreateRuleRequest {
  name: string;
  trigger: Trigger;
  action: Action;
  time_range?: TimeRange | null;
  conditions?: Condition[];
}

// ── Script types ────────────────────────────────────────────────────────────

export type ScriptStepType =
  | 'set_state'
  | 'set_brightness'
  | 'set_temperature'
  | 'delay'
  | 'apply_scene'
  | 'call_script';

export interface ScriptStep {
  type: ScriptStepType;
  device_name?: string;
  state?: string;
  brightness?: number | string;
  temperature?: number | string;
  milliseconds?: number;
  scene_name?: string;
  script_name?: string;
  args?: Record<string, unknown>;
}

export interface ScriptParam {
  name: string;
  description?: string;
  default?: unknown;
}

export interface Script {
  id: string;
  name: string;
  description?: string;
  params?: ScriptParam[];
  steps: ScriptStep[];
}

export interface CreateScriptRequest {
  name: string;
  description?: string;
  params?: ScriptParam[];
  steps: ScriptStep[];
}

export interface RunScriptRequest {
  args?: Record<string, unknown>;
}

export interface RunScriptResponse {
  script_id: string;
  status: string;
}

// ── Scene types ─────────────────────────────────────────────────────────────

export interface SceneState {
  state?: DeviceState;
  brightness?: number;
  temperature?: number;
}

export interface Scene {
  id: string;
  name: string;
  states: Record<string, SceneState>;
}

export interface CreateSceneRequest {
  name: string;
  states: Record<string, SceneState>;
}

export interface SnapshotSceneRequest {
  name: string;
  device_ids: string[];
}

export interface ApplySceneResponse {
  applied: number;
  errors: string[];
}

export interface SystemStatus {
  devices: number;
  clients: number;
  rules: number;
}

export type EventKind =
  | 'request'
  | 'device_connected'
  | 'device_disconnected'
  | 'device_updated'
  | 'device_error'
  | 'client_connected'
  | 'client_disconnected'
  | 'automation'
  | 'server';

export interface ServerEvent {
  event_id: string;
  timestamp: string;
  kind: EventKind;
  entity: string;
  message: string;
  device_name?: string;
  client_id?: string;
}

// ── Presence types ─────────────────────────────────────────────────────────────

export type PresenceState = 'home' | 'away' | 'unknown';
export type SourceState = 'home' | 'away' | 'unknown';

export interface Person {
  id: string;
  name: string;
  grace_period_secs: number;
  effective_state: PresenceState;
  sources: Record<string, SourceState>;
}

export interface CreatePersonRequest {
  name: string;
  grace_period_secs?: number;
}

export interface UpdateSourceRequest {
  state: SourceState;
}

// ── Dashboard types ───────────────────────────────────────────────────────────

export type CardType = 'entity_card' | 'gauge_card' | 'button_card' | 'stat_card' | 'history_card';

export type CardContent =
  | { card_type: 'entity_card'; entity_id: string }
  | { card_type: 'gauge_card'; entity_id: string; min: number; max: number; unit?: string }
  | { card_type: 'button_card'; entity_id: string; action: string }
  | { card_type: 'stat_card'; title: string; entity_ids: string[]; aggregation: string }
  | { card_type: 'history_card'; entity_id: string; hours: number };

export type Card = CardContent & { id: string; title?: string };

export interface View {
  id: string;
  title: string;
  icon?: string;
  cards: Card[];
}

export interface Dashboard {
  id: string;
  name: string;
  icon?: string;
  views: View[];
  created_at: string;
}

export interface CreateDashboardRequest {
  name: string;
  icon?: string;
}

export interface CreateViewRequest {
  title: string;
  icon?: string;
}

export type CreateCardRequest = CardContent;
