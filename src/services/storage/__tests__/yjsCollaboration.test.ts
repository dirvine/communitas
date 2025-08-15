import { describe, test, expect, beforeEach, afterEach } from '@jest/globals'
import * as Y from 'yjs'
import { WebrtcProvider } from 'y-webrtc'
import { IndexeddbPersistence } from 'y-indexeddb'
import { YjsMarkdownEditor } from '../yjsCollaboration'

// Mock WebRTC and IndexedDB for testing
jest.mock('y-webrtc')
jest.mock('y-indexeddb')

describe('YjsMarkdownEditor', () => {
  let editor1: YjsMarkdownEditor
  let editor2: YjsMarkdownEditor
  
  beforeEach(() => {
    editor1 = new YjsMarkdownEditor('user1', 'test-room')
    editor2 = new YjsMarkdownEditor('user2', 'test-room')
  })

  afterEach(async () => {
    await editor1.destroy()
    await editor2.destroy()
  })

  describe('Basic Yjs operations', () => {
    test('should sync text between two Y.Doc instances', async () => {
      // Given: Two Yjs documents
      await editor1.connect()
      await editor2.connect()
      
      // When: User A types "Hello", User B types "World"
      editor1.insertText(0, 'Hello ')
      editor2.insertText(6, 'World')
      
      // Simulate sync
      await editor1.syncWithPeer(editor2)
      
      // Then: Both documents contain "Hello World"
      expect(editor1.getContent()).toBe('Hello World')
      expect(editor2.getContent()).toBe('Hello World')
    })

    test('should handle concurrent edits without conflicts', async () => {
      // Given: Two users editing same line
      await editor1.connect()
      await editor2.connect()
      editor1.insertText(0, 'The quick brown fox')
      await editor1.syncWithPeer(editor2)
      
      // When: Both insert text at same position
      editor1.insertText(4, 'very ')
      editor2.insertText(4, 'super ')
      
      await editor1.syncWithPeer(editor2)
      
      // Then: Both changes preserved via CRDT
      const content = editor1.getContent()
      expect(content).toContain('very')
      expect(content).toContain('super')
      expect(editor2.getContent()).toBe(content)
    })

    test('should maintain cursor positions with awareness', async () => {
      // Given: Multiple users with cursors
      await editor1.connect()
      await editor2.connect()
      
      editor1.setCursorPosition(5)
      editor2.setCursorPosition(10)
      
      // When: Document changes
      editor1.insertText(0, 'PREFIX ')
      await editor1.syncWithPeer(editor2)
      
      // Then: Cursor positions update correctly
      expect(editor1.getCursorPosition()).toBe(12) // 5 + 7
      expect(editor2.getCursorPosition()).toBe(17) // 10 + 7
    })
  })

  describe('Conflict resolution', () => {
    test('should merge divergent document states', async () => {
      // Given: Two documents edited offline
      editor1.insertText(0, 'Line 1\n')
      editor2.insertText(0, 'Line A\n')
      
      // Simulate offline editing
      editor1.insertText(7, 'Line 2\n')
      editor2.insertText(7, 'Line B\n')
      
      // When: Reconnect and sync
      await editor1.connect()
      await editor2.connect()
      await editor1.syncWithPeer(editor2)
      
      // Then: All changes merged correctly
      const content = editor1.getContent()
      expect(content).toContain('Line 1')
      expect(content).toContain('Line 2')
      expect(content).toContain('Line A')
      expect(content).toContain('Line B')
      expect(editor2.getContent()).toBe(content)
    })

    test('should preserve formatting in concurrent edits', async () => {
      // Given: Markdown with bold, italic, links
      await editor1.connect()
      await editor2.connect()
      
      editor1.insertText(0, '# Title\n**bold** *italic* [link](url)')
      await editor1.syncWithPeer(editor2)
      
      // When: Multiple users edit
      editor1.insertText(8, '\n## Subtitle')
      editor2.insertText(15, 'very ')
      
      await editor1.syncWithPeer(editor2)
      
      // Then: Formatting maintained
      const content = editor1.getContent()
      expect(content).toContain('# Title')
      expect(content).toContain('## Subtitle')
      expect(content).toContain('**very bold**')
      expect(content).toContain('*italic*')
      expect(content).toContain('[link](url)')
    })
  })

  describe('Snapshot and history', () => {
    test('should create and restore snapshots', async () => {
      // Given: Document with edit history
      editor1.insertText(0, 'Version 1')
      const snapshot1 = await editor1.createSnapshot()
      
      editor1.insertText(9, ' - Modified')
      const snapshot2 = await editor1.createSnapshot()
      
      editor1.insertText(20, ' - Further modified')
      
      // When: Restore to snapshot
      await editor1.restoreSnapshot(snapshot1)
      
      // Then: Document state matches snapshot
      expect(editor1.getContent()).toBe('Version 1')
      
      // Can restore to different snapshot
      await editor1.restoreSnapshot(snapshot2)
      expect(editor1.getContent()).toBe('Version 1 - Modified')
    })

    test('should compute accurate diffs between snapshots', async () => {
      // Given: Two snapshots
      editor1.insertText(0, 'Line 1\nLine 2\nLine 3')
      const snapshot1 = await editor1.createSnapshot()
      
      editor1.replaceText('Line 2', 'Modified Line 2')
      editor1.insertText(editor1.getContent().length, '\nLine 4')
      const snapshot2 = await editor1.createSnapshot()
      
      // When: Calculate diff
      const diff = await editor1.diff(snapshot1, snapshot2)
      
      // Then: Shows exact changes
      expect(diff.additions).toContain('Modified Line 2')
      expect(diff.additions).toContain('Line 4')
      expect(diff.deletions).toContain('Line 2')
    })
  })

  describe('Collaboration features', () => {
    test('should track user presence and selections', async () => {
      await editor1.connect()
      await editor2.connect()
      
      editor1.setUserInfo({ name: 'Alice', color: '#ff0000' })
      editor2.setUserInfo({ name: 'Bob', color: '#0000ff' })
      
      editor1.setSelection(0, 5)
      editor2.setSelection(10, 15)
      
      await editor1.syncWithPeer(editor2)
      
      const users = editor1.getOnlineUsers()
      expect(users).toHaveLength(2)
      expect(users.find(u => u.name === 'Alice')).toBeDefined()
      expect(users.find(u => u.name === 'Bob')).toBeDefined()
    })

    test('should handle large collaborative sessions', async () => {
      // Simulate 10 users
      const editors: YjsMarkdownEditor[] = []
      for (let i = 0; i < 10; i++) {
        const editor = new YjsMarkdownEditor(`user${i}`, 'large-room')
        await editor.connect()
        editors.push(editor)
      }
      
      // Each user adds content
      for (let i = 0; i < 10; i++) {
        editors[i].insertText(0, `User ${i} content\n`)
      }
      
      // Sync all
      for (let i = 0; i < 10; i++) {
        for (let j = i + 1; j < 10; j++) {
          await editors[i].syncWithPeer(editors[j])
        }
      }
      
      // All should have same content
      const finalContent = editors[0].getContent()
      for (let i = 1; i < 10; i++) {
        expect(editors[i].getContent()).toBe(finalContent)
      }
      
      // Cleanup
      for (const editor of editors) {
        await editor.destroy()
      }
    })
  })

  describe('Performance', () => {
    test('should handle large documents efficiently', async () => {
      // Create large document (1MB of text)
      const largeText = 'Lorem ipsum '.repeat(100000)
      
      const startTime = performance.now()
      editor1.insertText(0, largeText)
      const insertTime = performance.now() - startTime
      
      expect(insertTime).toBeLessThan(100) // Should be very fast
      
      // Test sync performance
      const syncStart = performance.now()
      await editor1.syncWithPeer(editor2)
      const syncTime = performance.now() - syncStart
      
      expect(syncTime).toBeLessThan(500) // Sync should be reasonably fast
    })

    test('should efficiently handle rapid edits', async () => {
      await editor1.connect()
      
      const startTime = performance.now()
      
      // Simulate rapid typing (100 characters)
      for (let i = 0; i < 100; i++) {
        editor1.insertText(i, String.fromCharCode(65 + (i % 26)))
      }
      
      const endTime = performance.now()
      
      expect(endTime - startTime).toBeLessThan(50) // Should handle rapid edits
      expect(editor1.getContent().length).toBe(100)
    })
  })
})