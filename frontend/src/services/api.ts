export const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000'

export type Investigation = { id: string; title: string; description: string; status: string; created_at: string; owner_id: string }
export type Agent = { id: string; hostname: string; platform: string; version: string; last_seen: string }
export type Job = { id: string; investigation_id: string; agent_id: string; capability: string; status: string; created_at: string; completed_at: string | null; error: string | null }
export type Evidence = { id: string; investigation_id: string; job_id: string; capability: string; sha256: string; size_bytes: number; collected_at: string; details: Record<string, string> }
export type Report = { id: string; investigation_id: string; title: string; created_at: string; content: { evidence_count: number; evidence: { id: string; capability: string; sha256: string }[] } }

export class ApiError extends Error {
  constructor(message: string, public status: number) { super(message) }
}

async function request<T>(path: string, init: RequestInit = {}, token?: string): Promise<T> {
  const response = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...init.headers,
    },
  })
  if (!response.ok) {
    const body = await response.json().catch(() => ({}))
    throw new ApiError(body.detail || `Request failed (${response.status})`, response.status)
  }
  return response.json() as Promise<T>
}

export const api = {
  login: (email: string, password: string) => request<{ access_token: string }>('/api/auth/login', { method: 'POST', body: JSON.stringify({ email, password }) }),
  investigations: (token: string) => request<Investigation[]>('/api/investigations', {}, token),
  createInvestigation: (token: string, title: string, description: string) => request<Investigation>('/api/investigations', { method: 'POST', body: JSON.stringify({ title, description }) }, token),
  updateInvestigation: (token: string, id: string, status: 'open' | 'closed') => request<Investigation>(`/api/investigations/${id}`, { method: 'PATCH', body: JSON.stringify({ status }) }, token),
  agents: (token: string) => request<Agent[]>('/api/agents', {}, token),
  jobs: (token: string) => request<Job[]>('/api/jobs', {}, token),
  createJob: (token: string, investigation_id: string, agent_id: string, capability: string) => request<Job>('/api/jobs', { method: 'POST', body: JSON.stringify({ investigation_id, agent_id, capability }) }, token),
  evidence: (token: string) => request<Evidence[]>('/api/evidence', {}, token),
  evidenceContent: (token: string, id: string) => request<unknown>(`/api/evidence/${id}/content`, {}, token),
  reports: (token: string) => request<Report[]>('/api/reports', {}, token),
  createReport: (token: string, investigation_id: string, title: string) => request<Report>('/api/reports', { method: 'POST', body: JSON.stringify({ investigation_id, title }) }, token),
}
