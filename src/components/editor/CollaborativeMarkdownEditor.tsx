import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react'
import * as monaco from 'monaco-editor'
import { Box, Paper, IconButton, Toolbar, Typography, Chip, Avatar, Tooltip, Menu, MenuItem, Divider } from '@mui/material'
import {
  Save,
  Preview,
  Fullscreen,
  FullscreenExit,
  Settings,
  History,
  Share
} from '@mui/icons-material'
import { YjsMarkdownEditor } from '../../services/storage/yjsCollaboration'
import { MarkdownWebPublisher } from '../../services/storage/markdownPublisher'
import { NetworkIdentity } from '../../types/collaboration'
import { inputSanitizer } from '../../services/security/inputSanitization'
import { marked } from 'marked'

interface CollaborationCursor {
  userId: string
  username: string
  color: string
  position: monaco.Position
  selection?: monaco.Range
}

interface EditorUser {
  id: string
  name: string
  avatar?: string
  color: string
  isOnline: boolean
  lastSeen: number
}

interface CollaborativeMarkdownEditorProps {
  filePath: string
  entityId: string
  currentUser: NetworkIdentity
  publisher?: MarkdownWebPublisher
  onSave?: (content: string) => Promise<void>
  onPublish?: () => Promise<void>
  initialContent?: string
  readOnly?: boolean
  theme?: 'light' | 'dark' | 'auto'
  showPreview?: boolean
  showCollaborators?: boolean
  enableVersionHistory?: boolean
  className?: string
}

export const CollaborativeMarkdownEditor: React.FC<CollaborativeMarkdownEditorProps> = ({
  filePath,
  entityId,
  currentUser,
  publisher,
  onSave,
  onPublish,
  initialContent = '',
  readOnly = false,
  theme = 'auto',
  showPreview = true,
  showCollaborators = true,
  enableVersionHistory = true,
  className
}) => {
  const editorContainerRef = useRef<HTMLDivElement>(null)
  const previewContainerRef = useRef<HTMLDivElement>(null)
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null)
  const yjsEditorRef = useRef<YjsMarkdownEditor | null>(null)
  
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [viewMode, setViewMode] = useState<'editor' | 'preview' | 'split'>('split')
  const [isSaving, setIsSaving] = useState(false)
  const [lastSaved, setLastSaved] = useState<Date | null>(null)
  const [collaborators, setCollaborators] = useState<EditorUser[]>([])
  const [cursors, setCursors] = useState<CollaborationCursor[]>([])
  const [content, setContent] = useState(initialContent)
  const [renderedHtml, setRenderedHtml] = useState('')
  const [settingsAnchor, setSettingsAnchor] = useState<null | HTMLElement>(null)
  const [historyAnchor, setHistoryAnchor] = useState<null | HTMLElement>(null)

  // Color palette for collaborators
  const collaboratorColors = useMemo(() => [
    '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', 
    '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F'
  ], [])

  // Initialize Monaco Editor
  useEffect(() => {
    if (!editorContainerRef.current) return

    // Configure Monaco for markdown
    monaco.languages.register({ id: 'markdown' })
    monaco.languages.setMonarchTokensProvider('markdown', {
      tokenizer: {
        root: [
          [/^#{1,6}\s.*$/, 'markup.heading'],
          [/^\s*[-*+]\s/, 'markup.list'],
          [/^\s*\d+\.\s/, 'markup.list'],
          [/\*\*([^*]+)\*\*/, 'markup.bold'],
          [/\*([^*]+)\*/, 'markup.italic'],
          [/`([^`]+)`/, 'markup.inline.raw'],
          [/```[\s\S]*?```/, 'markup.raw'],
          [/\[([^\]]+)\]\(([^)]+)\)/, 'markup.underline.link'],
          [/^>.*$/, 'markup.quote'],
          [/^---+$/, 'markup.heading'],
        ]
      }
    })

    // Create editor
    const editor = monaco.editor.create(editorContainerRef.current, {
      value: content,
      language: 'markdown',
      theme: theme === 'dark' ? 'vs-dark' : 'vs',
      automaticLayout: true,
      wordWrap: 'on',
      lineNumbers: 'on',
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      fontSize: 14,
      fontFamily: "'Fira Code', 'Monaco', 'Menlo', 'Ubuntu Mono', monospace",
      readOnly,
      renderValidationDecorations: 'on',
      scrollbar: {
        vertical: 'visible',
        horizontal: 'visible',
        useShadows: false,
        verticalHasArrows: true,
        horizontalHasArrows: true,
      },
    })

    editorRef.current = editor

    // Auto-save on content change
    const contentChangeDisposable = editor.onDidChangeModelContent(() => {
      const rawContent = editor.getValue()
      
      // SECURITY: Validate content before processing
      try {
        const sanitizationResult = inputSanitizer.sanitizeMarkdown(rawContent, { maxLength: 100000 })
        
        // Log security warnings but don't block editing
        if (sanitizationResult.warnings.length > 0) {
          console.warn('Content sanitization warnings:', sanitizationResult.warnings)
        }
        
        // Only save if content is valid
        if (sanitizationResult.isValid) {
          setContent(sanitizationResult.sanitizedValue)
          
          if (onSave && !readOnly) {
            // Debounced save
            const saveTimeout = setTimeout(() => {
              handleSave(sanitizationResult.sanitizedValue)
            }, 2000)
            
            return () => clearTimeout(saveTimeout)
          }
        } else {
          // Show validation errors to user
          console.error('Content validation failed:', sanitizationResult.errors)
        }
      } catch (error) {
        console.error('Content sanitization error:', error)
      }
    })

    // Cursor position tracking for collaboration
    const cursorPositionDisposable = editor.onDidChangeCursorPosition((e) => {
      if (yjsEditorRef.current) {
        yjsEditorRef.current.setCursorPosition(e.position.column)
      }
    })

    return () => {
      contentChangeDisposable.dispose()
      cursorPositionDisposable.dispose()
      editor.dispose()
    }
  }, [])

  // Initialize Yjs collaboration
  useEffect(() => {
    const initCollaboration = async () => {
      if (!editorRef.current || readOnly) return

      try {
        const roomId = `${entityId}:${filePath}`
        const yjsEditor = new YjsMarkdownEditor(currentUser.fourWords, roomId)
        
        await yjsEditor.connect()
        yjsEditorRef.current = yjsEditor

        // Sync initial content
        if (initialContent) {
          // Replace content via low-level operations
          yjsEditor.replaceText(yjsEditor.getContent(), initialContent)
        }

        // Set user info for collaboration
        yjsEditor.setUserInfo({
          name: currentUser.fourWords, // Use four-word address as display name
          color: collaboratorColors[0] // Current user gets first color
        })

        // Listen for remote changes and sync with Monaco
        const unsubscribeContent = yjsEditor.onContentChange((newContent) => {
          if (editorRef.current) {
            const currentPosition = editorRef.current.getPosition()
            const currentContent = editorRef.current.getValue()

            // Only update if content actually changed to avoid loops
            if (newContent !== currentContent) {
              editorRef.current.setValue(newContent)
              if (currentPosition) {
                editorRef.current.setPosition(currentPosition)
              }
              setContent(newContent)
            }
          }
        })

        // Listen for collaborator updates
        const unsubscribeUserJoin = yjsEditor.onUserJoin((user) => {
          console.log('User joined:', user.name)
          updateCollaborators()
        })

        const unsubscribeUserLeave = yjsEditor.onUserLeave((userId) => {
          console.log('User left:', userId)
          updateCollaborators()
        })

        // Function to update collaborators list
        const updateCollaborators = () => {
          const users = yjsEditor.getOnlineUsers()
          const editorUsers: EditorUser[] = users.map((user, index) => ({
            id: user.id,
            name: user.name,
            color: collaboratorColors[index % collaboratorColors.length],
            isOnline: user.isOnline,
            lastSeen: Date.now()
          }))
          setCollaborators(editorUsers)
        }

        // Initial collaborator update
        updateCollaborators()

        // Set up periodic collaborator updates
        const collaboratorInterval = setInterval(updateCollaborators, 5000)

        // Listen for cursor updates from other users
        const updateCursors = () => {
          const users = yjsEditor.getOnlineUsers()
          const collaborationCursors: CollaborationCursor[] = users
            .filter(user => user.cursor !== undefined && user.id !== currentUser.fourWords)
            .map(user => ({
              userId: user.id,
              username: user.name,
              color: collaboratorColors[users.findIndex(u => u.id === user.id) % collaboratorColors.length],
              position: new monaco.Position(
                Math.max(1, Math.floor((user.cursor || 0) / 100) + 1), // Estimate line from cursor position
                Math.max(1, (user.cursor || 0) % 100 + 1) // Estimate column
              ),
              selection: user.selection ? new monaco.Range(
                Math.max(1, Math.floor(user.selection.start / 100) + 1),
                Math.max(1, user.selection.start % 100 + 1),
                Math.max(1, Math.floor(user.selection.end / 100) + 1),
                Math.max(1, user.selection.end % 100 + 1)
              ) : undefined
            }))

          setCursors(collaborationCursors)
        }

        // Update cursors periodically
        const cursorInterval = setInterval(updateCursors, 1000)
        updateCursors() // Initial update

        // Cleanup function
        return () => {
          clearInterval(collaboratorInterval)
          clearInterval(cursorInterval)
          unsubscribeContent()
          unsubscribeUserJoin()
          unsubscribeUserLeave()
        }

      } catch (error) {
        console.error('Failed to initialize collaboration:', error)
      }
    }

    initCollaboration()

    return () => {
      if (yjsEditorRef.current) {
        yjsEditorRef.current.destroy()
      }
    }
  }, [entityId, filePath, currentUser.fourWords, readOnly, initialContent])

  // Render cursors and selections
  useEffect(() => {
    if (!editorRef.current) return

    const decorations: string[] = []
    
    cursors.forEach(cursor => {
      // Add cursor decoration
      const cursorDecoration = editorRef.current!.createDecorationsCollection([{
        range: new monaco.Range(
          cursor.position.lineNumber,
          cursor.position.column,
          cursor.position.lineNumber,
          cursor.position.column + 1
        ),
        options: {
          className: 'collaboration-cursor',
          stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
          afterContentClassName: 'collaboration-cursor-label',
        }
      }])

      // Add selection decoration if exists
      if (cursor.selection) {
        const selectionDecoration = editorRef.current!.createDecorationsCollection([{
          range: cursor.selection,
          options: {
            className: 'collaboration-selection',
            stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
          }
        }])
        decorations.push(...selectionDecoration.getRanges().map(() => ''))
      }

      decorations.push(...cursorDecoration.getRanges().map(() => ''))
    })

    return () => {
      if (editorRef.current) {
        // Clear all decorations by setting empty collections
        editorRef.current.createDecorationsCollection([])
      }
    }
  }, [cursors])

  // Render markdown preview
  useEffect(() => {
    const renderPreview = async () => {
      if (!content) return

      try {
        let html: string
        if (publisher) {
          html = await publisher.markdownToHtml(content)
        } else {
          // Fallback local rendering when no publisher provided
          const sanitizationResult = inputSanitizer.sanitizeMarkdown(content)
          if (!sanitizationResult.isValid) {
            throw new Error(`Invalid markdown: ${sanitizationResult.errors.join(', ')}`)
          }
          const sanitizedMarkdown = sanitizationResult.sanitizedValue
          marked.setOptions({ gfm: true, breaks: true })
          const rawHtml = marked.parse(sanitizedMarkdown) as string
          html = inputSanitizer.sanitizeHTML(rawHtml)
        }
        setRenderedHtml(html)
      } catch (error) {
        console.error('Failed to render markdown:', error)
        setRenderedHtml('<p>Failed to render preview</p>')
      }
    }

    if (viewMode === 'preview' || viewMode === 'split') {
      renderPreview()
    }
  }, [content, publisher, viewMode])

  // Handlers
  const handleSave = useCallback(async (contentToSave?: string) => {
    if (!onSave || readOnly) return

    setIsSaving(true)
    try {
      await onSave(contentToSave || content)
      setLastSaved(new Date())
    } catch (error) {
      console.error('Save failed:', error)
    } finally {
      setIsSaving(false)
    }
  }, [onSave, content, readOnly])

  const handlePublish = useCallback(async () => {
    if (!onPublish) return

    try {
      await onPublish()
    } catch (error) {
      console.error('Publish failed:', error)
    }
  }, [onPublish])

  const toggleFullscreen = useCallback(() => {
    setIsFullscreen(prev => !prev)
  }, [])

  const handleViewModeChange = useCallback((mode: 'editor' | 'preview' | 'split') => {
    setViewMode(mode)
  }, [])

  const getViewModeStyles = useMemo(() => {
    switch (viewMode) {
      case 'editor':
        return { editor: '100%', preview: '0%' }
      case 'preview':
        return { editor: '0%', preview: '100%' }
      case 'split':
      default:
        return { editor: '50%', preview: '50%' }
    }
  }, [viewMode])

  return (
    <Paper 
      className={className}
      sx={{
        height: isFullscreen ? '100vh' : '600px',
        display: 'flex',
        flexDirection: 'column',
        position: isFullscreen ? 'fixed' : 'relative',
        top: isFullscreen ? 0 : 'auto',
        left: isFullscreen ? 0 : 'auto',
        right: isFullscreen ? 0 : 'auto',
        bottom: isFullscreen ? 0 : 'auto',
        zIndex: isFullscreen ? 9999 : 1,
        overflow: 'hidden'
      }}
    >
      {/* Toolbar */}
      <Toolbar
        variant="dense"
        sx={{
          borderBottom: 1,
          borderColor: 'divider',
          minHeight: '48px !important',
          px: 2
        }}
      >
        <Typography variant="h6" sx={{ flexGrow: 1, fontSize: '1rem' }}>
          {filePath}
        </Typography>

        {/* Save Status */}
        {lastSaved && (
          <Chip
            size="small"
            label={`Saved ${lastSaved.toLocaleTimeString()}`}
            color="success"
            variant="outlined"
            sx={{ mr: 2 }}
          />
        )}

        {/* Collaborators */}
        {showCollaborators && collaborators.length > 0 && (
          <Box sx={{ display: 'flex', mr: 2 }}>
            {collaborators.slice(0, 3).map((collaborator) => (
              <Tooltip key={collaborator.id} title={collaborator.name}>
                <Avatar
                  sx={{
                    width: 24,
                    height: 24,
                    fontSize: '0.75rem',
                    bgcolor: collaborator.color,
                    ml: -0.5,
                    border: '1px solid white'
                  }}
                >
                  {collaborator.name[0]}
                </Avatar>
              </Tooltip>
            ))}
            {collaborators.length > 3 && (
              <Tooltip title={`+${collaborators.length - 3} more`}>
                <Avatar
                  sx={{
                    width: 24,
                    height: 24,
                    fontSize: '0.6rem',
                    bgcolor: 'grey.400',
                    ml: -0.5,
                    border: '1px solid white'
                  }}
                >
                  +{collaborators.length - 3}
                </Avatar>
              </Tooltip>
            )}
          </Box>
        )}

        {/* View Mode Controls */}
        <Box sx={{ display: 'flex', mr: 1 }}>
          <Tooltip title="Editor Only">
            <IconButton
              size="small"
              onClick={() => handleViewModeChange('editor')}
              color={viewMode === 'editor' ? 'primary' : 'default'}
            >
              <span style={{ fontSize: '14px', fontWeight: 'bold' }}>{ '{}'}</span>
            </IconButton>
          </Tooltip>
          
          <Tooltip title="Split View">
            <IconButton
              size="small"
              onClick={() => handleViewModeChange('split')}
              color={viewMode === 'split' ? 'primary' : 'default'}
            >
              <span style={{ fontWeight: 700 }}>II</span>
            </IconButton>
          </Tooltip>
          
          <Tooltip title="Preview Only">
            <IconButton
              size="small"
              onClick={() => handleViewModeChange('preview')}
              color={viewMode === 'preview' ? 'primary' : 'default'}
            >
              <Preview />
            </IconButton>
          </Tooltip>
        </Box>

        {/* Action Buttons */}
        {!readOnly && (
          <Tooltip title="Save">
            <IconButton
              size="small"
              onClick={() => handleSave()}
              disabled={isSaving}
              sx={{ mr: 0.5 }}
            >
              <Save />
            </IconButton>
          </Tooltip>
        )}

        {onPublish && (
          <Tooltip title="Publish">
            <IconButton
              size="small"
              onClick={handlePublish}
              sx={{ mr: 0.5 }}
            >
              <Share />
            </IconButton>
          </Tooltip>
        )}

        {enableVersionHistory && (
          <Tooltip title="Version History">
            <IconButton
              size="small"
              onClick={(e) => setHistoryAnchor(e.currentTarget)}
              sx={{ mr: 0.5 }}
            >
              <History />
            </IconButton>
          </Tooltip>
        )}

        <Tooltip title="Settings">
          <IconButton
            size="small"
            onClick={(e) => setSettingsAnchor(e.currentTarget)}
            sx={{ mr: 0.5 }}
          >
            <Settings />
          </IconButton>
        </Tooltip>

        <Tooltip title={isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}>
          <IconButton size="small" onClick={toggleFullscreen}>
            {isFullscreen ? <FullscreenExit /> : <Fullscreen />}
          </IconButton>
        </Tooltip>
      </Toolbar>

      {/* Editor Content */}
      <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Monaco Editor */}
        <Box
          ref={editorContainerRef}
          sx={{
            width: getViewModeStyles.editor,
            transition: 'width 0.3s ease',
            display: viewMode === 'preview' ? 'none' : 'block'
          }}
        />

        {/* Divider */}
        {viewMode === 'split' && (
          <Divider orientation="vertical" />
        )}

        {/* Preview Pane */}
        {showPreview && (
          <Box
            ref={previewContainerRef}
            sx={{
              width: getViewModeStyles.preview,
              transition: 'width 0.3s ease',
              overflow: 'auto',
              p: 2,
              display: viewMode === 'editor' ? 'none' : 'block',
              '& h1, & h2, & h3, & h4, & h5, & h6': {
                borderBottom: '1px solid',
                borderColor: 'divider',
                pb: 0.5,
                mb: 2
              },
              '& pre': {
                backgroundColor: 'grey.100',
                p: 2,
                borderRadius: 1,
                overflow: 'auto'
              },
              '& blockquote': {
                borderLeft: '4px solid',
                borderColor: 'primary.main',
                pl: 2,
                ml: 0,
                fontStyle: 'italic'
              }
            }}
            dangerouslySetInnerHTML={{ __html: renderedHtml }}
          />
        )}
      </Box>

      {/* Settings Menu */}
      <Menu
        anchorEl={settingsAnchor}
        open={Boolean(settingsAnchor)}
        onClose={() => setSettingsAnchor(null)}
      >
        <MenuItem onClick={() => setSettingsAnchor(null)}>
          Theme Settings
        </MenuItem>
        <MenuItem onClick={() => setSettingsAnchor(null)}>
          Editor Preferences
        </MenuItem>
        <MenuItem onClick={() => setSettingsAnchor(null)}>
          Collaboration Settings
        </MenuItem>
      </Menu>

      {/* Version History Menu */}
      <Menu
        anchorEl={historyAnchor}
        open={Boolean(historyAnchor)}
        onClose={() => setHistoryAnchor(null)}
      >
        <MenuItem onClick={() => setHistoryAnchor(null)}>
          View History
        </MenuItem>
        <MenuItem onClick={() => setHistoryAnchor(null)}>
          Compare Versions
        </MenuItem>
        <MenuItem onClick={() => setHistoryAnchor(null)}>
          Restore Version
        </MenuItem>
      </Menu>

      {/* Custom Styles for Collaboration */}
      <style>{`
        .collaboration-cursor {
          border-left: 2px solid var(--cursor-color);
          animation: blink 1s infinite;
        }
        
        .collaboration-cursor-label::after {
          content: attr(data-username);
          position: absolute;
          top: -20px;
          left: 0;
          background: var(--cursor-color);
          color: white;
          padding: 2px 6px;
          border-radius: 3px;
          font-size: 11px;
          white-space: nowrap;
        }
        
        .collaboration-selection {
          background-color: var(--selection-color);
          opacity: 0.3;
        }
        
        @keyframes blink {
          0%, 50% { opacity: 1; }
          51%, 100% { opacity: 0; }
        }
      `}</style>
    </Paper>
  )
}
