import React, { useState, useCallback } from 'react'
import {
  Box,
  Typography,
  Button,
  Card,
  CardContent,
  Grid,
  Chip,
  LinearProgress,
  IconButton,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
} from '@mui/material'
import {
  CloudUpload,
  Folder,
  InsertDriveFile,
  Image,
  VideoFile,
  AudioFile,
  PictureAsPdf,
  Download,
  Share,
  Delete,
  MoreVert,
} from '@mui/icons-material'

interface FileItem {
  id: string
  name: string
  type: string
  size: number
  uploadDate: Date
  sharedBy: string
  downloadUrl: string
  thumbnail?: string
  isUploading?: boolean
  uploadProgress?: number
}

interface FileShareInterfaceProps {
  groupId?: string
  onFileUpload: (files: File[]) => void
  onFileDownload: (file: FileItem) => void
  onFileDelete: (fileId: string) => void
  onFileShare: (fileId: string, recipients: string[]) => void
}

const mockFiles: FileItem[] = [
  {
    id: '1',
    name: 'Project Proposal.pdf',
    type: 'application/pdf',
    size: 2048000,
    uploadDate: new Date(2025, 0, 5),
    sharedBy: 'Alice Johnson',
    downloadUrl: '#',
  },
  {
    id: '2',
    name: 'Team Photo.jpg',
    type: 'image/jpeg',
    size: 512000,
    uploadDate: new Date(2025, 0, 4),
    sharedBy: 'Bob Smith',
    downloadUrl: '#',
    thumbnail: '/placeholder-image.jpg',
  },
  {
    id: '3',
    name: 'Meeting Recording.mp4',
    type: 'video/mp4',
    size: 15728640,
    uploadDate: new Date(2025, 0, 3),
    sharedBy: 'Carol Williams',
    downloadUrl: '#',
    isUploading: true,
    uploadProgress: 67,
  },
]

export default function FileShareInterface({
  groupId,
  onFileUpload,
  onFileDownload,
  onFileDelete,
  onFileShare,
}: FileShareInterfaceProps) {
  const [files, setFiles] = useState<FileItem[]>(mockFiles)
  const [dragOver, setDragOver] = useState(false)
  const [menuAnchor, setMenuAnchor] = useState<HTMLElement | null>(null)
  const [selectedFile, setSelectedFile] = useState<FileItem | null>(null)
  const [shareDialog, setShareDialog] = useState(false)
  const [shareRecipients, setShareRecipients] = useState('')
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid')

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(false)
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(false)
    
    const droppedFiles = Array.from(e.dataTransfer.files)
    if (droppedFiles.length > 0) {
      onFileUpload(droppedFiles)
      
      // Add optimistic updates
      const newFiles = droppedFiles.map(file => ({
        id: 'temp-' + Date.now() + Math.random(),
        name: file.name,
        type: file.type,
        size: file.size,
        uploadDate: new Date(),
        sharedBy: 'You',
        downloadUrl: '',
        isUploading: true,
        uploadProgress: 0,
      }))
      
      setFiles(prev => [...newFiles, ...prev])
    }
  }, [onFileUpload])

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = e.target.files
    if (selectedFiles) {
      const fileArray = Array.from(selectedFiles)
      onFileUpload(fileArray)
      
      // Reset input
      e.target.value = ''
    }
  }, [onFileUpload])

  const formatFileSize = (bytes: number) => {
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    if (bytes === 0) return '0 Bytes'
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i]
  }

  const getFileIcon = (type: string) => {
    if (type.startsWith('image/')) return <Image color="primary" />
    if (type.startsWith('video/')) return <VideoFile color="secondary" />
    if (type.startsWith('audio/')) return <AudioFile color="success" />
    if (type === 'application/pdf') return <PictureAsPdf color="error" />
    if (type.includes('folder')) return <Folder color="warning" />
    return <InsertDriveFile color="action" />
  }

  const handleMenuOpen = (event: React.MouseEvent, file: FileItem) => {
    setMenuAnchor(event.currentTarget as HTMLElement)
    setSelectedFile(file)
  }

  const handleMenuClose = () => {
    setMenuAnchor(null)
    setSelectedFile(null)
  }

  const handleShare = () => {
    setShareDialog(true)
    handleMenuClose()
  }

  const handleShareConfirm = () => {
    if (selectedFile && shareRecipients) {
      const recipients = shareRecipients.split(',').map(r => r.trim())
      onFileShare(selectedFile.id, recipients)
      setShareDialog(false)
      setShareRecipients('')
    }
  }

  const handleDownload = (file: FileItem) => {
    onFileDownload(file)
    handleMenuClose()
  }

  const handleDelete = (file: FileItem) => {
    onFileDelete(file.id)
    setFiles(prev => prev.filter(f => f.id !== file.id))
    handleMenuClose()
  }

  const filteredFiles = files
  const totalSize = files.reduce((acc, f) => acc + f.size, 0)
  const uploadingCount = files.filter(f => f.isUploading).length

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Card sx={{ mb: 2 }}>
        <CardContent>
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
            <Typography variant="h6">
              Shared Files {groupId && ('- Group ' + groupId)}
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <Button
                variant="outlined"
                onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}
              >
                {viewMode === 'grid' ? 'List' : 'Grid'}
              </Button>
              <Button
                variant="contained"
                component="label"
                startIcon={<CloudUpload />}
              >
                Upload Files
                <input
                  type="file"
                  hidden
                  multiple
                  onChange={handleFileSelect}
                  accept="image/*,video/*,audio/*,.pdf,.doc,.docx,.txt,.zip"
                />
              </Button>
            </Box>
          </Box>

          <Box sx={{ display: 'flex', gap: 2 }}>
            <Chip 
              label={files.length + ' files'}
              variant="outlined" 
            />
            <Chip 
              label={formatFileSize(totalSize) + ' total'}
              variant="outlined" 
            />
            <Chip 
              label={uploadingCount + ' uploading'}
              color="info" 
              variant="outlined" 
            />
          </Box>
        </CardContent>
      </Card>

      <Box
        sx={{
          flexGrow: 1,
          position: 'relative',
          border: dragOver ? '2px dashed #1976d2' : '2px dashed transparent',
          borderRadius: 2,
          backgroundColor: dragOver ? 'action.hover' : 'transparent',
          transition: 'all 0.2s ease',
          overflow: 'auto',
        }}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {dragOver && (
          <Box
            sx={{
              position: 'absolute',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              backgroundColor: 'rgba(25, 118, 210, 0.1)',
              zIndex: 10,
              borderRadius: 2,
            }}
          >
            <Box sx={{ textAlign: 'center' }}>
              <CloudUpload sx={{ fontSize: 64, color: 'primary.main', mb: 2 }} />
              <Typography variant="h6" color="primary">
                Drop files here to upload
              </Typography>
            </Box>
          </Box>
        )}

        {filteredFiles.length === 0 ? (
          <Box sx={{ 
            display: 'flex', 
            flexDirection: 'column', 
            alignItems: 'center', 
            justifyContent: 'center',
            height: '400px',
            color: 'text.secondary' 
          }}>
            <Folder sx={{ fontSize: 64, mb: 2 }} />
            <Typography variant="h6" gutterBottom>
              No files shared yet
            </Typography>
            <Typography variant="body2">
              Drop files here or click upload to get started
            </Typography>
          </Box>
        ) : (
          <Grid container spacing={2} sx={{ p: 2 }}>
            {filteredFiles.map((file) => (
              <Grid 
                item 
                xs={12} 
                sm={viewMode === 'grid' ? 6 : 12} 
                md={viewMode === 'grid' ? 4 : 12} 
                lg={viewMode === 'grid' ? 3 : 12}
                key={file.id}
              >
                <Card
                  sx={{
                    position: 'relative',
                    cursor: 'pointer',
                    '&:hover': { elevation: 4 },
                    opacity: file.isUploading ? 0.7 : 1,
                  }}
                >
                  <CardContent sx={{ p: 2 }}>
                    <Box sx={{ display: 'flex', alignItems: 'flex-start', gap: 2 }}>
                      <Box sx={{ minWidth: 48 }}>
                        {file.thumbnail ? (
                          <img 
                            src={file.thumbnail} 
                            alt={file.name}
                            style={{ 
                              width: 48, 
                              height: 48, 
                              objectFit: 'cover', 
                              borderRadius: 4 
                            }}
                          />
                        ) : (
                          <Box sx={{ 
                            width: 48, 
                            height: 48, 
                            display: 'flex', 
                            alignItems: 'center', 
                            justifyContent: 'center',
                            backgroundColor: 'grey.100',
                            borderRadius: 1,
                          }}>
                            {getFileIcon(file.type)}
                          </Box>
                        )}
                      </Box>

                      <Box sx={{ flexGrow: 1, minWidth: 0 }}>
                        <Typography variant="subtitle1" noWrap title={file.name}>
                          {file.name}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {formatFileSize(file.size)}  •  {file.sharedBy}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {file.uploadDate.toLocaleDateString()}
                        </Typography>

                        {file.isUploading && (
                          <Box sx={{ mt: 1 }}>
                            <LinearProgress 
                              variant="determinate" 
                              value={file.uploadProgress || 0} 
                              sx={{ mb: 0.5 }}
                            />
                            <Typography variant="caption">
                              Uploading... {file.uploadProgress || 0}%
                            </Typography>
                          </Box>
                        )}
                      </Box>

                      <IconButton
                        size="small"
                        onClick={(e) => handleMenuOpen(e, file)}
                        disabled={file.isUploading}
                      >
                        <MoreVert fontSize="small" />
                      </IconButton>
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>
        )}
      </Box>

      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={handleMenuClose}
      >
        <MenuItem onClick={() => selectedFile && handleDownload(selectedFile)}>
          <Download sx={{ mr: 1 }} />
          Download
        </MenuItem>
        <MenuItem onClick={handleShare}>
          <Share sx={{ mr: 1 }} />
          Share
        </MenuItem>
        <MenuItem onClick={() => selectedFile && handleDelete(selectedFile)}>
          <Delete sx={{ mr: 1 }} />
          Delete
        </MenuItem>
      </Menu>

      <Dialog open={shareDialog} onClose={() => setShareDialog(false)}>
        <DialogTitle>Share File</DialogTitle>
        <DialogContent>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            Share "{selectedFile?.name}" with:
          </Typography>
          <TextField
            fullWidth
            label="Recipients (comma-separated addresses)"
            value={shareRecipients}
            onChange={(e) => setShareRecipients(e.target.value)}
            placeholder="calm-river-mountain-dawn, bright-star-ocean-wind"
            helperText="Enter four-word addresses separated by commas"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setShareDialog(false)}>Cancel</Button>
          <Button variant="contained" onClick={handleShareConfirm}>
            Share
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  )
}
