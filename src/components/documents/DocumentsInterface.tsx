import { useState, useCallback } from 'react'
import { Box, IconButton, Tooltip } from '@mui/material'
import { ArrowBack } from '@mui/icons-material'
import DocumentList from './DocumentList'
import CollaborativeEditor from './CollaborativeEditor'

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

interface DocumentsInterfaceProps {
  initialDocuments?: DocumentInfo[]
}

export default function DocumentsInterface({
  initialDocuments = [],
}: DocumentsInterfaceProps) {
  const [documents, setDocuments] = useState<DocumentInfo[]>(initialDocuments)
  const [selectedDocument, setSelectedDocument] = useState<DocumentInfo | null>(null)
  const [isEditorMode, setIsEditorMode] = useState(false)

  const handleDocumentSelect = useCallback((document: DocumentInfo) => {
    setSelectedDocument(document)
    setIsEditorMode(true)
  }, [])

  const handleDocumentCreate = useCallback((title: string) => {
    console.log('Creating document:', title)
  }, [])

  const handleDocumentDelete = useCallback((id: string) => {
    setDocuments(prev => prev.filter(doc => doc.id !== id))
    
    if (selectedDocument?.id === id) {
      setSelectedDocument(null)
      setIsEditorMode(false)
    }
  }, [selectedDocument])

  const handleBackToList = useCallback(() => {
    setIsEditorMode(false)
    setSelectedDocument(null)
  }, [])

  const handleDocumentSave = useCallback((content: string) => {
    if (selectedDocument) {
      setDocuments(prev => 
        prev.map(doc => 
          doc.id === selectedDocument.id 
            ? { ...doc, content, lastModified: new Date() }
            : doc
        )
      )
      
      setSelectedDocument(prev => 
        prev ? { ...prev, content, lastModified: new Date() } : null
      )
    }
  }, [selectedDocument])

  const handleContentChange = useCallback((content: string) => {
    console.log('Content changed:', content.length, 'characters')
  }, [])

  if (isEditorMode && selectedDocument) {
    return (
      <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
        <Box sx={{ 
          display: { xs: 'flex', lg: 'none' },
          p: 1,
          borderBottom: 1,
          borderColor: 'divider',
          alignItems: 'center',
          gap: 1,
        }}>
          <Tooltip title="Back to documents">
            <IconButton onClick={handleBackToList}>
              <ArrowBack />
            </IconButton>
          </Tooltip>
        </Box>

        <CollaborativeEditor
          
          title={selectedDocument.title}
          initialContent={selectedDocument.content}
          permissions={selectedDocument.permissions}
          onSave={handleDocumentSave}
          onContentChange={handleContentChange}
          readonly={selectedDocument.permissions === 'view'}
        />
      </Box>
    )
  }

  return (
    <DocumentList
      documents={documents}
      onDocumentSelect={handleDocumentSelect}
      onDocumentCreate={handleDocumentCreate}
      onDocumentDelete={handleDocumentDelete}
    />
  )
}

export type { DocumentInfo }
