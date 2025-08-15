import React, { useState } from 'react'
import {
  Box,
  Card,
  CardContent,
  Typography,
  Button,
  IconButton,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Grid,
  Chip,
  Avatar,
  AvatarGroup,
  InputAdornment,
  Fab,
} from '@mui/material'
import {
  Add,
  Search,
  MoreVert,
  Edit,
  Share,
  Delete,
  Description,
  Group,
  AccessTime,
} from '@mui/icons-material'

interface DocumentInfo {
  id: string
  title: string
  content: string
  lastModified: Date
  author: string
  collaborators: Array<{
    id: string
    name: string
    avatar: string
  }>
  isShared: boolean
  permissions: 'view' | 'comment' | 'edit' | 'admin'
}

interface DocumentListProps {
  documents: DocumentInfo[]
  onDocumentSelect: (document: DocumentInfo) => void
  onDocumentCreate: (title: string) => void
  onDocumentDelete: (id: string) => void
}

const mockDocuments: DocumentInfo[] = [
  {
    id: '1',
    title: 'Project Proposal Draft',
    content: '# Project Proposal\n\n## Overview\nThis document outlines...',
    lastModified: new Date(Date.now() - 3600000),
    author: 'Alice Johnson',
    collaborators: [
      { id: '1', name: 'Alice Johnson', avatar: '/avatar1.jpg' },
      { id: '2', name: 'Bob Smith', avatar: '/avatar2.jpg' },
    ],
    isShared: true,
    permissions: 'edit',
  },
  {
    id: '2',
    title: 'Meeting Notes - January 15',
    content: '# Meeting Notes\n\n## Attendees\n- Alice\n- Bob\n- Carol\n\n## Action Items...',
    lastModified: new Date(Date.now() - 86400000),
    author: 'Bob Smith',
    collaborators: [
      { id: '2', name: 'Bob Smith', avatar: '/avatar2.jpg' },
      { id: '3', name: 'Carol Williams', avatar: '/avatar3.jpg' },
    ],
    isShared: true,
    permissions: 'edit',
  },
  {
    id: '3',
    title: 'Personal Notes',
    content: '# Personal Notes\n\nThings to remember...',
    lastModified: new Date(Date.now() - 172800000),
    author: 'You',
    collaborators: [{ id: 'me', name: 'You', avatar: '/avatar-me.jpg' }],
    isShared: false,
    permissions: 'admin',
  },
]

export default function DocumentList({
  documents: propDocuments,
  onDocumentSelect,
  onDocumentCreate,
  onDocumentDelete,
}: DocumentListProps) {
  const [documents, setDocuments] = useState<DocumentInfo[]>(propDocuments.length > 0 ? propDocuments : mockDocuments)
  const [searchQuery, setSearchQuery] = useState('')
  const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null)
  const [selectedDocument, setSelectedDocument] = useState<DocumentInfo | null>(null)
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [newDocumentTitle, setNewDocumentTitle] = useState('')

  const filteredDocuments = documents.filter(doc =>
    doc.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    doc.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
    doc.author.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const handleMenuOpen = (event: React.MouseEvent, document: DocumentInfo) => {
    event.stopPropagation()
    setMenuAnchor(event.currentTarget as HTMLElement)
    setSelectedDocument(document)
  }

  const handleMenuClose = () => {
    setMenuAnchor(null)
    setSelectedDocument(null)
  }

  const handleDocumentClick = (document: DocumentInfo) => {
    onDocumentSelect(document)
  }

  const handleCreateDocument = () => {
    if (newDocumentTitle.trim()) {
      const newDocument: DocumentInfo = {
        id: Date.now().toString(),
        title: newDocumentTitle.trim(),
        content: `# ${newDocumentTitle.trim()}\n\nStart writing here...`,
        lastModified: new Date(),
        author: 'You',
        collaborators: [{ id: 'me', name: 'You', avatar: '/avatar-me.jpg' }],
        isShared: false,
        permissions: 'admin',
      }
      
      setDocuments(prev => [newDocument, ...prev])
      onDocumentCreate(newDocumentTitle.trim())
      setCreateDialogOpen(false)
      setNewDocumentTitle('')
      
      onDocumentSelect(newDocument)
    }
  }

  const handleDeleteDocument = () => {
    if (selectedDocument) {
      setDocuments(prev => prev.filter(doc => doc.id !== selectedDocument.id))
      onDocumentDelete(selectedDocument.id)
      handleMenuClose()
    }
  }

  const handleShareDocument = () => {
    console.log('Sharing document:', selectedDocument?.title)
    handleMenuClose()
  }

  const formatRelativeTime = (date: Date) => {
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
    const diffDays = Math.floor(diffHours / 24)
    
    if (diffHours < 1) return 'Just now'
    if (diffHours < 24) return `${diffHours} hours ago`
    if (diffDays === 1) return 'Yesterday'
    if (diffDays < 7) return `${diffDays} days ago`
    return date.toLocaleDateString()
  }

  const getPreviewText = (content: string, maxLength: number = 150) => {
    const plainText = content.replace(/^#+\s/gm, '').replace(/\*\*(.*?)\*\*/g, '$1').replace(/\*(.*?)\*/g, '$1')
    if (plainText.length <= maxLength) return plainText
    return plainText.substring(0, maxLength) + '...'
  }

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <Box sx={{ p: 3, borderBottom: 1, borderColor: 'divider' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
          <Typography variant="h5" component="h1" sx={{ fontWeight: 600 }}>
            Documents
          </Typography>
          <Button
            variant="contained"
            startIcon={<Add />}
            onClick={() => setCreateDialogOpen(true)}
          >
            New Document
          </Button>
        </Box>

        <TextField
          fullWidth
          placeholder="Search documents..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <Search color="action" />
              </InputAdornment>
            ),
          }}
          sx={{ maxWidth: 500 }}
        />
      </Box>

      {/* Document Grid */}
      <Box sx={{ flexGrow: 1, p: 3, overflow: 'auto' }}>
        {filteredDocuments.length === 0 ? (
          <Box sx={{ 
            display: 'flex', 
            flexDirection: 'column', 
            alignItems: 'center', 
            justifyContent: 'center',
            height: '400px',
            color: 'text.secondary' 
          }}>
            <Description sx={{ fontSize: 64, mb: 2 }} />
            <Typography variant="h6" gutterBottom>
              {documents.length === 0 ? 'No documents yet' : 'No documents found'}
            </Typography>
            <Typography variant="body2" sx={{ mb: 3 }}>
              {documents.length === 0 
                ? 'Create your first document to get started with collaborative editing'
                : 'Try adjusting your search terms'
              }
            </Typography>
            {documents.length === 0 && (
              <Button
                variant="contained"
                startIcon={<Add />}
                onClick={() => setCreateDialogOpen(true)}
              >
                Create Document
              </Button>
            )}
          </Box>
        ) : (
          <Grid container spacing={3}>
            {filteredDocuments.map((document) => (
              <Grid item xs={12} sm={6} md={4} lg={3} key={document.id}>
                <Card
                  sx={{
                    cursor: 'pointer',
                    transition: 'all 0.2s ease',
                    '&:hover': {
                      elevation: 4,
                      transform: 'translateY(-2px)',
                    },
                    height: '100%',
                    display: 'flex',
                    flexDirection: 'column',
                  }}
                  onClick={() => handleDocumentClick(document)}
                >
                  <CardContent sx={{ flexGrow: 1, p: 2 }}>
                    <Box sx={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', mb: 2 }}>
                      <Typography variant="h6" component="h3" sx={{ 
                        fontWeight: 600, 
                        flexGrow: 1,
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                        pr: 1,
                      }}>
                        {document.title}
                      </Typography>
                      <IconButton
                        size="small"
                        onClick={(e) => handleMenuOpen(e, document)}
                        sx={{ ml: 1 }}
                      >
                        <MoreVert fontSize="small" />
                      </IconButton>
                    </Box>

                    <Typography
                      variant="body2"
                      color="text.secondary"
                      sx={{
                        mb: 2,
                        height: 60,
                        overflow: 'hidden',
                        display: '-webkit-box',
                        WebkitLineClamp: 3,
                        WebkitBoxOrient: 'vertical',
                        lineHeight: 1.4,
                      }}
                    >
                      {getPreviewText(document.content)}
                    </Typography>

                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
                      <AccessTime sx={{ fontSize: 16, color: 'text.secondary' }} />
                      <Typography variant="caption" color="text.secondary">
                        {formatRelativeTime(document.lastModified)}
                      </Typography>
                      {document.isShared && (
                        <Chip
                          icon={<Group />}
                          label="Shared"
                          size="small"
                          variant="outlined"
                          sx={{ ml: 'auto' }}
                        />
                      )}
                    </Box>

                    <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                      <AvatarGroup max={3} sx={{ flexGrow: 1 }}>
                        {document.collaborators.map((collaborator) => (
                          <Avatar
                            key={collaborator.id}
                            src={collaborator.avatar}
                            sx={{ width: 24, height: 24 }}
                            title={collaborator.name}
                          >
                            {collaborator.name[0]}
                          </Avatar>
                        ))}
                      </AvatarGroup>
                      
                      <Typography variant="caption" color="text.secondary">
                        by {document.author}
                      </Typography>
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        )}
      </Box>

      <Fab
        color="primary"
        sx={{
          position: 'fixed',
          bottom: 16,
          right: 16,
          display: { xs: 'flex', sm: 'none' },
        }}
        onClick={() => setCreateDialogOpen(true)}
      >
        <Add />
      </Fab>

      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={handleMenuClose}
      >
        <MenuItem onClick={() => selectedDocument && handleDocumentClick(selectedDocument)}>
          <Edit sx={{ mr: 1 }} />
          Edit
        </MenuItem>
        <MenuItem onClick={handleShareDocument}>
          <Share sx={{ mr: 1 }} />
          Share
        </MenuItem>
        <MenuItem onClick={handleDeleteDocument} sx={{ color: 'error.main' }}>
          <Delete sx={{ mr: 1 }} />
          Delete
        </MenuItem>
      </Menu>

      <Dialog open={createDialogOpen} onClose={() => setCreateDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create New Document</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Document Title"
            fullWidth
            variant="outlined"
            value={newDocumentTitle}
            onChange={(e) => setNewDocumentTitle(e.target.value)}
            placeholder="Enter document title..."
            onKeyDown={(e) => {
              if (e.key === 'Enter' && newDocumentTitle.trim()) {
                handleCreateDocument()
              }
            }}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button
            variant="contained"
            onClick={handleCreateDocument}
            disabled={!newDocumentTitle.trim()}
          >
            Create
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
