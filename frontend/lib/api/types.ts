export type DeviceType = 'light' | 'thermostat' | 'lock' | 'switch' | 'sensor';
export type DeviceState = 'on' | 'off' | 'unknown';

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
  service_type: string;
  properties: Record<string, string>;
  suggested_type: DeviceType;
}

export interface Trigger {
  type: 'state_change' | 'temperature_above' | 'temperature_below';
  device_name: string;
  target_state?: DeviceState;
  threshold?: number;
}

export interface Action {
  type: 'state' | 'brightness' | 'temperature';
  device_name: string;
  state?: DeviceState;
  brightness?: number;
  temperature?: number;
}

export interface AutomationRule {
  name: string;
  enabled: boolean;
  trigger: Trigger;
  action: Action;
}

export interface CreateRuleRequest {
  name: string;
  trigger: Trigger;
  action: Action;
}

export interface SystemStatus {
  devices: number;
  clients: number;
  rules: number;
}
