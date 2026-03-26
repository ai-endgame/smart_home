export function createEventSource(): EventSource {
  return new EventSource('/api/events/stream');
}
