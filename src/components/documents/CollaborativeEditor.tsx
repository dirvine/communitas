import { useState, useEffect, useCallback, useRef } from 'react'
import {
  Box,
  Paper,
  Typography,
  Button,
  IconButton,
  Chip,
  Avatar,
  AvatarGroup,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  FormControl,
  InputLabel,
  Select,
  Divider,
  Tooltip,
  Badge,
} from '@mui/material'
import {
  Save,
  Share,
  History,
  People,
  Undo,
  Redo,
  Comment,
  MoreVert,
  Lock,
  Public,
  Group,
} from '@mui/icons-material'
import Editor from '@monaco-editor/react'
type OnMount = (editor: editor.IStandaloneCodeEditor, monaco: typeof import('monaco-editor')) => void
type OnChange = (value?: string) => void
import type { editor } from 'monaco-editor'

interface CollaboratorInfo {
  id: string
  name: string
  avatar: string
  color: string
  cursor?: {
    line: number
    column: number
  }
  selection?: {
    startLine: number
    startColumn: number
    endLine: number
    endColumn: number
  }
  isActive: boolean
  lastSeen: Date
}

interface DocumentVersion {
  id: string
  timestamp: Date
  author: string
  message: string
  changes: number
}

interface CollaborativeEditorProps {
  // _documentId: string
  initialContent?: string
  title: string
  onSave?: (content: string) => void
  onContentChange?: (content: string) => void
  readonly?: boolean
  permissions?: 'view' | 'comment' | 'edit' | 'admin'
}

// Mock data for demonstration
const mockCollaborators: CollaboratorInfo[] = [
  {
    id: '1',
    name: 'Alice Johnson',
    avatar: '/avatar1.jpg',
    color: '#FF6B6B',
    cursor: { line: 5, column: 12 },
    isActive: true,
    lastSeen: new Date(),
  },
  {
    id: '2',
    name: 'Bob Smith',
    avatar: '/avatar2.jpg',
    color: '#4ECDC4',
    selection: { startLine: 10, startColumn: 0, endLine: 12, endColumn: 20 },
    isActive: true,
    lastSeen: new Date(Date.now() - 30000),
  },
  {
    id: '3',
    name: 'Carol Williams',
    avatar: '/avatar3.jpg',
    color: '#45B7D1',
    isActive: false,
    lastSeen: new Date(Date.now() - 300000),
  },
]

const mockVersions: DocumentVersion[] = [
  {
    id: '1',
    timestamp: new Date(Date.now() - 3600000),
    author: 'Alice Johnson',
    message: 'Initial draft with basic structure',
    changes: 234,
  },
  {
    id: '2',
    timestamp: new Date(Date.now() - 1800000),
    author: 'Bob Smith',
    message: 'Added introduction and methodology section',
    changes: 89,
  },
  {
    id: '3',
    timestamp: new Date(Date.now() - 600000),
    author: 'Alice Johnson',
    message: 'Fixed typos and improved formatting',
    changes: 12,
  },
]

export default function CollaborativeEditor({
  // _documentId,
  initialContent = '',
  title,
  onSave,
  onContentChange,
  readonly = false,
  permissions = 'edit',
}: CollaborativeEditorProps) {
  const [content, setContent] = useState(initialContent)
  const [collaborators] = useState<CollaboratorInfo[]>(mockCollaborators)
  const [versions, setVersions] = useState<DocumentVersion[]>(mockVersions)
  const [hasUnsavedChanges, setHasUnsavedChanges] = useState(false)
  const [shareMenuAnchor, setShareMenuAnchor] = useState<HTMLElement | null>(null)
  const [versionHistoryOpen, setVersionHistoryOpen] = useState(false)
  const [collaboratorsOpen, setCollaboratorsOpen] = useState(false)
  const [shareDialogOpen, setShareDialogOpen] = useState(false)
  const [shareEmail, setShareEmail] = useState('')
  const [sharePermission, setSharePermission] = useState('edit')
  const [comments] = useState<number>(3)
  
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const monacoRef = useRef<typeof import('monaco-editor') | null>(null)

  // Handle editor mount
  const handleEditorDidMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor
    monacoRef.current = monaco

    // Configure editor theme
    monaco.editor.defineTheme('communitas-light', {
      base: 'vs',
      inherit: true,
      rules: [
        { token: 'comment', foreground: '6A9955' },
        { token: 'keyword', foreground: '0077AA' },
        { token: 'string', foreground: 'A31515' },
      ],
      colors: {
        'editor.background': '#FFFFFF',
        'editor.lineHighlightBackground': '#F5F5F5',
      }
    })

    monaco.editor.setTheme('communitas-light')

    // Track cursor and selection changes
    editor.onDidChangeCursorPosition((e) => {
      console.log('Cursor moved to:', e.position)
    })

    editor.onDidChangeCursorSelection((e) => {
      console.log('Selection changed:', e.selection)
    })

  }, [collaborators])

  // Handle content changes
  const handleContentChange: OnChange = useCallback((value = '') => {
    setContent(value)
    setHasUnsavedChanges(true)
    onContentChange?.(value)
    console.log('Content changed, length:', value.length)
  }, [onContentChange])

  // Save document
  const handleSave = useCallback(() => {
    onSave?.(content)
    setHasUnsavedChanges(false)
    
    const newVersion: DocumentVersion = {
      id: Date.now().toString(),
      timestamp: new Date(),
      author: 'You',
      message: 'Manual save',
      changes: content.length,
    }
    
    setVersions(prev => [newVersion, ...prev])
    console.log('Document saved')
  }, [content, onSave])

  // Auto-save functionality
  useEffect(() => {
    if (hasUnsavedChanges) {
      const autoSaveTimer = setTimeout(() => {
        handleSave()
        console.log('Auto-saved document')
      }, 30000)

      return () => clearTimeout(autoSaveTimer)
    }
  }, [hasUnsavedChanges, handleSave])

  // Handle sharing
  const handleShare = useCallback(() => {
    setShareDialogOpen(true)
    setShareMenuAnchor(null)
  }, [])

  const handleShareConfirm = useCallback(() => {
    console.log('Sharing document with:', shareEmail, 'permission:', sharePermission)
    setShareDialogOpen(false)
    setShareEmail('')
    setSharePermission('edit')
  }, [shareEmail, sharePermission])

  // Undo/Redo actions
  const handleUndo = () => editorRef.current?.trigger('keyboard', 'undo', null)
  const handleRedo = () => editorRef.current?.trigger('keyboard', 'redo', null)

  const activeCollaborators = collaborators.filter(c => c.isActive)
  const canEdit = permissions === 'edit' || permissions === 'admin'

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Paper elevation={1} sx={{ p: 2, borderRadius: 0 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="h6" component="h1" sx={{ fontWeight: 500 }}>
              {title}
            </Typography>
            {hasUnsavedChanges && (
              <Chip label="Unsaved" size="small" color="warning" />
            )}
            <Chip 
              label={`${collaborators.length} collaborators`} 
              size="small" 
              variant="outlined"
              onClick={() => setCollaboratorsOpen(true)}
            />
          </Box>

          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {/* Active collaborators */}
            <AvatarGroup max={4} sx={{ mr: 2 }}>
              {activeCollaborators.map((collaborator) => (
                <Tooltip key={collaborator.id} title={collaborator.name}>
                  <Badge
                    color="success"
                    variant="dot"
                    invisible={!collaborator.isActive}
                  >
                    <Avatar
                      src={collaborator.avatar}
                      sx={{
                        width: 32,
                        height: 32,
                        border: `2px solid ${collaborator.color}`,
                      }}
                    >
                      {collaborator.name[0]}
                    </Avatar>
                  </Badge>
                </Tooltip>
              ))}
            </AvatarGroup>

            {/* Comments */}
            <Tooltip title="Comments">
              <IconButton size="small">
                <Badge badgeContent={comments} color="primary">
                  <Comment />
                </Badge>
              </IconButton>
            </Tooltip>

            {/* Action buttons */}
            {canEdit && (
              <>
                <Tooltip title="Undo">
                  <IconButton size="small" onClick={handleUndo}>
                    <Undo />
                  </IconButton>
                </Tooltip>
                <Tooltip title="Redo">
                  <IconButton size="small" onClick={handleRedo}>
                    <Redo />
                  </IconButton>
                </Tooltip>
                <Button
                  variant="contained"
                  startIcon={<Save />}
                  onClick={handleSave}
                  disabled={!hasUnsavedChanges}
                  size="small"
                >
                  Save
                </Button>
              </>
            )}

            <Tooltip title="Version History">
              <IconButton size="small" onClick={() => setVersionHistoryOpen(true)}>
                <History />
              </IconButton>
            </Tooltip>

            <Button
              variant="outlined"
              startIcon={<Share />}
              onClick={(e) => setShareMenuAnchor(e.currentTarget)}
              size="small"
            >
              Share
            </Button>
          </Box>
        </Box>
      </Paper>

      {/* Editor */}
      <Box sx={{ flexGrow: 1, position: 'relative' }}>
        <Editor
          height="100%"
          defaultLanguage="markdown"
          value={content}
          onChange={handleContentChange}
          onMount={handleEditorDidMount}
          options={{
            readOnly: readonly || !canEdit,
            minimap: { enabled: true },
            lineNumbers: 'on',
            wordWrap: 'on',
            fontSize: 14,
            fontFamily: '"SF Mono", Monaco, "Cascadia Code", "Roboto Mono", Consolas, "Courier New", monospace',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 2,
            renderWhitespace: 'selection',
            bracketPairColorization: { enabled: true },
            // suggest: true,
            quickSuggestions: true,
          }}
        />

        {/* Collaboration indicators overlay */}
        {activeCollaborators.map((collaborator) => (
          collaborator.isActive && (
            <Box
              key={collaborator.id}
              sx={{
                position: 'absolute',
                top: 8,
                right: 8,
                backgroundColor: collaborator.color,
                color: 'white',
                px: 1,
                py: 0.5,
                borderRadius: 1,
                fontSize: '0.75rem',
                fontWeight: 500,
                pointerEvents: 'none',
                zIndex: 10,
              }}
            >
              {collaborator.name} is editing
            </Box>
          )
        ))}
      </Box>

      {/* Share Menu */}
      <Menu
        anchorEl={shareMenuAnchor}
        open={Boolean(shareMenuAnchor)}
        onClose={() => setShareMenuAnchor(null)}
      >
        <MenuItem onClick={handleShare}>
          <People sx={{ mr: 1 }} />
          Invite people
        </MenuItem>
        <MenuItem onClick={() => setCollaboratorsOpen(true)}>
          <Group sx={{ mr: 1 }} />
          Manage collaborators
        </MenuItem>
        <Divider />
        <MenuItem>
          <Public sx={{ mr: 1 }} />
          Anyone with link can view
        </MenuItem>
        <MenuItem>
          <Lock sx={{ mr: 1 }} />
          Restricted access
        </MenuItem>
      </Menu>

      {/* Share Dialog */}
      <Dialog open={shareDialogOpen} onClose={() => setShareDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Share Document</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Email or four-word address"
            fullWidth
            variant="outlined"
            value={shareEmail}
            onChange={(e) => setShareEmail(e.target.value)}
            placeholder="alice@example.com or calm-river-mountain-dawn"
            sx={{ mb: 2 }}
          />
          
          <FormControl fullWidth>
            <InputLabel>Permission</InputLabel>
            <Select
              value={sharePermission}
              label="Permission"
              onChange={(e) => setSharePermission(e.target.value)}
            >
              <MenuItem value="view">Can view</MenuItem>
              <MenuItem value="comment">Can comment</MenuItem>
              <MenuItem value="edit">Can edit</MenuItem>
              {permissions === 'admin' && (
                <MenuItem value="admin">Admin access</MenuItem>
              )}
            </Select>
          </FormControl>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShareDialogOpen(false)}>Cancel</Button>
          <Button variant="contained" onClick={handleShareConfirm}>
            Send Invitation
          </Button>
        </DialogActions>
      </Dialog>

      {/* Version History Dialog */}
      <Dialog 
        open={versionHistoryOpen} 
        onClose={() => setVersionHistoryOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>Version History</DialogTitle>
        <DialogContent>
          {versions.map((version, index) => (
            <Box key={version.id} sx={{ mb: 2, pb: 2, borderBottom: index < versions.length - 1 ? 1 : 0, borderColor: 'divider' }}>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
                <Typography variant="subtitle2">{version.author}</Typography>
                <Typography variant="caption" color="text.secondary">
                  {version.timestamp.toLocaleString()}
                </Typography>
              </Box>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                {version.message}
              </Typography>
              <Typography variant="caption">
                {version.changes} characters changed
              </Typography>
            </Box>
          ))}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setVersionHistoryOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Collaborators Dialog */}
      <Dialog 
        open={collaboratorsOpen} 
        onClose={() => setCollaboratorsOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Collaborators</DialogTitle>
        <DialogContent>
          {collaborators.map((collaborator) => (
            <Box key={collaborator.id} sx={{ display: 'flex', alignItems: 'center', mb: 2, gap: 2 }}>
              <Badge
                color="success"
                variant="dot"
                invisible={!collaborator.isActive}
              >
                <Avatar src={collaborator.avatar} sx={{ border: `2px solid ${collaborator.color}` }}>
                  {collaborator.name[0]}
                </Avatar>
              </Badge>
              <Box sx={{ flexGrow: 1 }}>
                <Typography variant="subtitle2">{collaborator.name}</Typography>
                <Typography variant="caption" color="text.secondary">
                  {collaborator.isActive ? 'Active now' : 'Last seen ' + collaborator.lastSeen.toLocaleString()}
                </Typography>
              </Box>
              {permissions === 'admin' && (
                <IconButton size="small">
                  <MoreVert />
                </IconButton>
              )}
            </Box>
          ))}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCollaboratorsOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
