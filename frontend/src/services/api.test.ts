import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from './api'

afterEach(() => vi.unstubAllGlobals())

describe('investigation API', () => {
  it('sends the bearer token for case creation', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => ({ id: 'case-1' }) })
    vi.stubGlobal('fetch', fetchMock)
    await api.createInvestigation('token-1', 'Case title', 'Notes')
    expect(fetchMock).toHaveBeenCalledWith(expect.stringContaining('/api/investigations'), expect.objectContaining({
      method: 'POST',
      headers: expect.objectContaining({ Authorization: 'Bearer token-1' }),
    }))
  })

  it('reports API validation errors', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, status: 422, json: async () => ({ detail: 'Unsupported collection capability' }) }))
    await expect(api.createJob('token', 'case', 'agent', 'unsafe.call')).rejects.toThrow('Unsupported collection capability')
  })
})
