import { describe, it, expect, beforeEach, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
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

describe('EntityDirectoryContext - addExistingContact', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Clear localStorage to prevent state leaking between tests
    window.localStorage.clear()
  })

  describe('Four-Word Validation', () => {
    it('should validate four-word format before fetching from DHT', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      // Mock validation to succeed
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          return {
            id: 'dht-id-123',
            four_words: 'ocean-blue-eagle-star',
            display_name: 'Alice',
            public_key: 'pk_real_dht_key',
            dht_address: 'dht://oceanblueaglestar',
          }
        }
        return null
      })

      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'ocean-blue-eagle-star' })
      })

      // Should call validation first
      expect(invoke).toHaveBeenCalledWith('validate_four_words', {
        fourWords: 'ocean-blue-eagle-star',
      })

      // Then fetch from DHT
      expect(invoke).toHaveBeenCalledWith('core_fetch_identity', {
        fourWords: 'ocean-blue-eagle-star',
      })
    })

    it('should return error for invalid four-word format', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      // Mock local validation to fail
      const { validateFourWordIdentity } = await import('../../utils/identity')
      vi.mocked(validateFourWordIdentity).mockResolvedValueOnce(false)

      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'invalid-format' })
      })

      expect(response).toMatchObject({
        success: false,
        error: 'Invalid Four-Word format',
      })

      // Should NOT call core_fetch_identity
      expect(invoke).not.toHaveBeenCalledWith('core_fetch_identity', expect.anything())
    })

    it('should normalize four-word input (trim, lowercase, replace spaces)', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          return {
            id: 'dht-id-123',
            four_words: 'ocean-blue-eagle-star',
            display_name: 'Alice',
            public_key: 'pk_real_dht_key',
            dht_address: 'dht://oceanblueaglestar',
          }
        }
        return null
      })

      await act(async () => {
        await result.current.addExistingContact({ fourWords: '  Ocean Blue Eagle STAR  ' })
      })

      // Should normalize before validation
      expect(invoke).toHaveBeenCalledWith('validate_four_words', {
        fourWords: 'ocean-blue-eagle-star',
      })
    })
  })

  describe('DHT Identity Fetching', () => {
    it('should fetch identity from DHT using invoke("core_fetch_identity")', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      const mockDHTIdentity = {
        id: 'dht-id-456',
        four_words: 'mountain-river-forest-cloud',
        display_name: 'Bob Smith',
        public_key: 'pk_dht_bob_key',
        dht_address: 'dht://mountainriverforestcloud',
        bio: 'Engineer at TechCorp',
        avatar_url: 'https://example.com/avatar.jpg',
      }

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') return mockDHTIdentity
        return null
      })

      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'mountain-river-forest-cloud' })
      })

      expect(invoke).toHaveBeenCalledWith('core_fetch_identity', {
        fourWords: 'mountain-river-forest-cloud',
      })
    })

    it('should handle DHT fetch errors gracefully', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          throw new Error('Identity not found in DHT')
        }
        return null
      })

      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'unknown-entity-test-words' })
      })

      expect(response).toMatchObject({
        success: false,
        error: 'Identity not found in DHT',
      })
    })

    it('should handle network timeout errors', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          throw new Error('Network timeout')
        }
        return null
      })

      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'ocean-blue-eagle-star' })
      })

      expect(response).toMatchObject({
        success: false,
        error: 'Network timeout',
      })
    })
  })

  describe('Contact Creation from DHT Data', () => {
    it('should create PersonalUser with real DHT data', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      const mockDHTIdentity = {
        id: 'dht-id-789',
        four_words: 'valley-peak-stream-stone',
        display_name: 'Carol Lee',
        public_key: 'pk_carol_dht_key',
        dht_address: 'dht://valleypeakstreamstone',
        bio: 'Designer at CreativeCo',
      }

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') return mockDHTIdentity
        return null
      })

      await act(async () => {
        await result.current.addExistingContact({ fourWords: 'valley-peak-stream-stone' })
      })

      const contacts = result.current.personalUsers

      expect(contacts).toHaveLength(1)
      expect(contacts[0]).toMatchObject({
        name: 'Carol Lee',
        description: 'Designer at CreativeCo',
        networkIdentity: {
          fourWords: 'valley-peak-stream-stone',
          publicKey: 'pk_carol_dht_key',
          dhtAddress: 'dht://valleypeakstreamstone',
          isOwned: false,
          isValidated: true,
        },
      })
    })

    it('should mark entity as syncStatus: "synced" (not "new")', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          return {
            id: 'dht-id-sync',
            four_words: 'test-sync-entity-words',
            display_name: 'Synced User',
            public_key: 'pk_synced',
            dht_address: 'dht://testsyncentitywords',
          }
        }
        return null
      })

      await act(async () => {
        await result.current.addExistingContact({ fourWords: 'test-sync-entity-words' })
      })

      const contact = result.current.personalUsers[0]
      expect(contact.syncStatus).toBe('synced')
      expect(contact.lastSyncedAt).toBeDefined()
    })

    it('should set isValidated: true and isOwned: false in networkIdentity', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          return {
            id: 'dht-id-validated',
            four_words: 'test-validated-contact-word',
            display_name: 'Validated User',
            public_key: 'pk_validated',
            dht_address: 'dht://testvalidatedcontactword',
          }
        }
        return null
      })

      await act(async () => {
        await result.current.addExistingContact({ fourWords: 'test-validated-contact-word' })
      })

      const contact = result.current.personalUsers[0]
      expect(contact.networkIdentity.isValidated).toBe(true)
      expect(contact.networkIdentity.isOwned).toBe(false)
    })

    it('should add contact to personalUsers state', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      expect(result.current.personalUsers).toHaveLength(0)

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          return {
            id: 'dht-id-add',
            four_words: 'add-to-state-test-words',
            display_name: 'State User',
            public_key: 'pk_state',
            dht_address: 'dht://addtostatetestwords',
          }
        }
        return null
      })

      await act(async () => {
        await result.current.addExistingContact({ fourWords: 'add-to-state-test-words' })
      })

      expect(result.current.personalUsers).toHaveLength(1)
    })

    it('should return success result with entityId', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          return {
            id: 'dht-id-success',
            four_words: 'success-result-test-words',
            display_name: 'Success User',
            public_key: 'pk_success',
            dht_address: 'dht://successresulttestwords',
          }
        }
        return null
      })

      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'success-result-test-words' })
      })

      expect(response).toMatchObject({
        success: true,
        entityId: expect.stringContaining('contact-'),
      })
    })
  })

  describe('Online/Offline Behavior', () => {
    it('should work only when online (throw error if offline)', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      // Simulate offline state by making invoke throw a network error
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') {
          throw new Error('Network unavailable - cannot fetch from DHT')
        }
        return null
      })

      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'offline-test-entity-words' })
      })

      expect(response).toMatchObject({
        success: false,
        error: 'Network unavailable - cannot fetch from DHT',
      })
    })
  })

  describe('Duplicate Prevention', () => {
    it('should prevent adding same contact twice by four-words', async () => {
      const { result } = renderHook(() => useEntityDirectory(), { wrapper })

      const mockDHTIdentity = {
        id: 'dht-id-duplicate',
        four_words: 'duplicate-test-entity-words',
        display_name: 'Duplicate User',
        public_key: 'pk_duplicate',
        dht_address: 'dht://duplicatetestentitywords',
      }

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === 'validate_four_words') return true
        if (cmd === 'core_fetch_identity') return mockDHTIdentity
        return null
      })

      // Add first time
      await act(async () => {
        await result.current.addExistingContact({ fourWords: 'duplicate-test-entity-words' })
      })

      expect(result.current.personalUsers).toHaveLength(1)

      // Try to add again
      let response
      await act(async () => {
        response = await result.current.addExistingContact({ fourWords: 'duplicate-test-entity-words' })
      })

      expect(response).toMatchObject({
        success: false,
        error: 'Contact already exists with these four-words',
      })

      // Should still have only one contact
      expect(result.current.personalUsers).toHaveLength(1)
    })
  })
})