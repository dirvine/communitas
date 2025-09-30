import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import '@testing-library/jest-dom'
import { invoke } from '@tauri-apps/api/core'
import { EntityDirectoryProvider, useEntityDirectory } from '../EntityDirectoryContext'
import React from 'react'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock nanoid for consistent test IDs
vi.mock('nanoid', () => ({
  nanoid: vi.fn(() => 'test1234'),
}))

// Mock identity validation
vi.mock('../../utils/identity', () => ({
  validateFourWordIdentity: vi.fn(() => Promise.resolve(true)),
}))

// Test wrapper component
const wrapper = ({ children }: { children: React.ReactNode }) => (
  <EntityDirectoryProvider currentUserId="test-user-id">
    {children}
  </EntityDirectoryProvider>
)

describe('EntityDirectoryContext - createContact', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    window.localStorage.clear()
  })

  describe('Offline Contact Creation', () => {
    it('should create contact with temp four-words when offline', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      // No mocks needed - should work offline
      let response
      await act(async () => {
        response = await result.current.createContact({
          displayName: 'Alice Smith',
          email: 'alice@example.com',
          relationship: 'colleague',
        })
      })

      expect(response).toMatchObject({
        success: true,
        isOwned: true,
        needsSync: true,
      })

      // Should generate temp four-words
      expect(response.fourWords).toMatch(/^temp-/)
    })

    it('should create contact with generated real four-words when online', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      // Mock saorsa-core to return real four-words
      vi.mocked(invoke).mockResolvedValueOnce('ocean-blue-eagle-star')

      let response
      await act(async () => {
        response = await result.current.createContact({
          displayName: 'Bob Johnson',
        })
      })

      expect(invoke).toHaveBeenCalledWith('generate_four_word_identity')

      expect(response).toMatchObject({
        success: true,
        fourWords: 'ocean-blue-eagle-star',
        isOwned: true,
        needsSync: true,
      })
    })

    it('should fallback to temp four-words if generation fails', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      // Mock generation to fail
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Network unavailable'))

      let response
      await act(async () => {
        response = await result.current.createContact({
          displayName: 'Carol Lee',
        })
      })

      expect(response).toMatchObject({
        success: true,
        isOwned: true,
        needsSync: true,
      })

      // Should fallback to temp
      expect(response.fourWords).toMatch(/^temp-/)
    })
  })

  describe('Contact State Management', () => {
    it('should add contact to personalUsers state', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      expect(result.current.personalUsers).toHaveLength(0)

      await act(async () => {
        await result.current.createContact({
          displayName: 'Test User',
        })
      })

      expect(result.current.personalUsers).toHaveLength(1)
      expect(result.current.personalUsers[0]).toMatchObject({
        name: 'Test User',
        relationship: 'colleague',
      })
    })

    it('should mark contact as syncStatus: "new"', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      await act(async () => {
        await result.current.createContact({
          displayName: 'New Contact',
        })
      })

      const contact = result.current.personalUsers[0]
      expect(contact.syncStatus).toBe('new')
      expect(contact.lastSyncedAt).toBeUndefined()
    })

    it('should set isValidated: false and isOwned: true in networkIdentity', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      await act(async () => {
        await result.current.createContact({
          displayName: 'Owned Contact',
        })
      })

      const contact = result.current.personalUsers[0]
      expect(contact.networkIdentity.isValidated).toBe(false)
      expect(contact.networkIdentity.isOwned).toBe(true)
    })

    it('should use provided displayName and relationship', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      await act(async () => {
        await result.current.createContact({
          displayName: 'David Chen',
          relationship: 'friend',
          email: 'david@example.com',
        })
      })

      const contact = result.current.personalUsers[0]
      expect(contact.name).toBe('David Chen')
      expect(contact.relationship).toBe('friend')
      expect(contact.description).toBe('david@example.com')
    })

    it('should default relationship to "colleague" if not provided', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      await act(async () => {
        await result.current.createContact({
          displayName: 'Default Relationship',
        })
      })

      const contact = result.current.personalUsers[0]
      expect(contact.relationship).toBe('colleague')
    })
  })

  describe('Sync Queue', () => {
    it('should queue contact for sync (needsSync: true)', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      let response
      await act(async () => {
        response = await result.current.createContact({
          displayName: 'Sync Test',
        })
      })

      expect(response.needsSync).toBe(true)
    })

    it('should add create operation to queue', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      await act(async () => {
        await result.current.createContact({
          displayName: 'Queue Test',
        })
      })

      // Check operations queue
      expect(result.current.operations).toHaveLength(1)
      expect(result.current.operations[0]).toMatchObject({
        entityType: 'contact',
        operation: 'create',
      })
    })
  })

  describe('Multiple Contacts', () => {
    it('should allow creating multiple contacts', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      await act(async () => {
        await result.current.createContact({ displayName: 'Contact 1' })
        await result.current.createContact({ displayName: 'Contact 2' })
        await result.current.createContact({ displayName: 'Contact 3' })
      })

      expect(result.current.personalUsers).toHaveLength(3)
      expect(result.current.personalUsers.map(c => c.name)).toEqual([
        'Contact 1',
        'Contact 2',
        'Contact 3',
      ])
    })

  })

  describe('Input Validation', () => {
    it('should trim whitespace from displayName', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      await act(async () => {
        await result.current.createContact({
          displayName: '  Whitespace Test  ',
        })
      })

      const contact = result.current.personalUsers[0]
      expect(contact.name).toBe('Whitespace Test')
    })

    it('should handle empty displayName gracefully', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      let response
      await act(async () => {
        response = await result.current.createContact({
          displayName: '',
        })
      })

      // Should still succeed but with empty name
      expect(response.success).toBe(true)
      expect(result.current.personalUsers[0].name).toBe('')
    })
  })
})