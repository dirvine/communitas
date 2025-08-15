import React, { useEffect, useState, useCallback, useRef, useMemo } from 'react'
import {
  Box,
  Paper,
  AppBar,
  Toolbar,
  IconButton,
  TextField,
  Typography,
  Breadcrumbs,
  Link,
  CircularProgress,
  Alert,
  Fab,
  Drawer,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Chip,
  InputAdornment,
  Menu,
  MenuItem,
  Tooltip
} from '@mui/material'
import {
  ArrowBack,
  ArrowForward,
  Home,
  Refresh,
  Search,
  Bookmark,
  BookmarkBorder,
  Share,
  Print,
  Download,
  Fullscreen,
  FullscreenExit,
  Menu as MenuIcon,
  Close,
  History,
  Language,
  Visibility,
  VisibilityOff
} from '@mui/icons-material'
import { CompleteStorageSystem } from '../../services/storage/CompleteStorageSystem'
import { MarkdownWebPublisher, TableOfContentsEntry } from '../../services/storage/markdownPublisher'
import { NetworkIdentity } from '../../types/collaboration'

interface NavigationEntry {
  url: string
  title: string
  timestamp: number
}

interface Bookmark {
  url: string
  title: string
  identity: string
  addedAt: number
}

interface SearchResult {
  title: string
  url: string
  snippet: string
  identity: string
  score: number
}

interface MarkdownBrowserProps {
  storageSystem: CompleteStorageSystem
  currentUser: NetworkIdentity
  initialUrl?: string
  theme?: 'light' | 'dark' | 'auto'
  showNavigationHistory?: boolean
  enableSearch?: boolean
  enableBookmarks?: boolean
  showTableOfContents?: boolean
  className?: string
}

export const MarkdownBrowser: React.FC<MarkdownBrowserProps> = ({
  storageSystem,
  currentUser,
  initialUrl = 'ocean-forest-moon-star/home.md',
  theme = 'auto',
  showNavigationHistory = true,
  enableSearch = true,
  enableBookmarks = true,
  showTableOfContents = true,
  className
}) => {
  const contentRef = useRef<HTMLDivElement>(null)
  
  const [currentUrl, setCurrentUrl] = useState(initialUrl)
  const [addressBarUrl, setAddressBarUrl] = useState(initialUrl)
  const [content, setContent] = useState('')
  const [title, setTitle] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [isFullscreen, setIsFullscreen] = useState(false)
  
  // Navigation
  const [navigationHistory, setNavigationHistory] = useState<NavigationEntry[]>([])
  const [historyIndex, setHistoryIndex] = useState(-1)
  const [canGoBack, setCanGoBack] = useState(false)
  const [canGoForward, setCanGoForward] = useState(false)
  
  // Table of Contents
  const [tableOfContents, setTableOfContents] = useState<TableOfContentsEntry[]>([])
  const [tocOpen, setTocOpen] = useState(false)
  
  // Search
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<SearchResult[]>([])
  const [searchOpen, setSearchOpen] = useState(false)
  
  // Bookmarks
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([])
  const [isBookmarked, setIsBookmarked] = useState(false)
  const [bookmarksOpen, setBookmarksOpen] = useState(false)
  
  // Menus
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null)
  
  // Publishers cache
  const [publishers] = useState(new Map<string, MarkdownWebPublisher>())

  // Parse URL to extract identity and path
  const parseUrl = useCallback((url: string) => {
    const match = url.match(/^([a-z]+-[a-z]+-[a-z]+-[a-z]+)(?:\/(.+))?$/)
    if (match) {
      return {
        identity: match[1],
        path: match[2] || 'home.md'
      }
    }
    return { identity: '', path: '' }
  }, [])

  // Get or create publisher for identity
  const getPublisher = useCallback(async (identity: string): Promise<MarkdownWebPublisher | null> => {
    if (publishers.has(identity)) {
      return publishers.get(identity)!
    }

    try {
      // Try to find entity by identity
      // This is simplified - in production would query DHT
      const mockEntityId = `entity_${identity}`
      const publisher = await storageSystem.getWebPublisher(mockEntityId)
      publishers.set(identity, publisher)
      return publisher
    } catch (error) {
      console.error(`Failed to get publisher for ${identity}:`, error)
      return null
    }
  }, [publishers, storageSystem])

  // Navigate to URL
  const navigateTo = useCallback(async (url: string, addToHistory = true) => {
    if (!url) return

    setLoading(true)
    setError(null)
    
    try {
      const { identity, path } = parseUrl(url)
      
      if (!identity) {
        throw new Error('Invalid URL format')
      }

      const publisher = await getPublisher(identity)
      if (!publisher) {
        throw new Error(`Cannot connect to ${identity}`)
      }

      // Load content
      let content: string
      let pageTitle: string
      
      try {
        content = await publisher.getProcessedContent(`/web/${path}`)
        pageTitle = extractTitle(content) || path
      } catch (contentError) {
        // If specific file not found, try to load home.md
        if (path !== 'home.md') {
          content = await publisher.getProcessedContent('/web/home.md')
          pageTitle = extractTitle(content) || 'Home'
        } else {
          throw contentError
        }
      }

      // Update state
      setCurrentUrl(url)
      setAddressBarUrl(url)
      setContent(content)
      setTitle(pageTitle)

      // Generate table of contents
      if (showTableOfContents) {
        const toc = await publisher.generateTableOfContents(content)
        setTableOfContents(toc)
      }

      // Add to navigation history
      if (addToHistory) {
        const entry: NavigationEntry = {
          url,
          title: pageTitle,
          timestamp: Date.now()
        }
        
        setNavigationHistory(prev => {
          const newHistory = [...prev.slice(0, historyIndex + 1), entry]
          return newHistory.slice(-50) // Keep only last 50 entries
        })
        
        setHistoryIndex(prev => prev + 1)
      }

      // Update navigation buttons
      updateNavigationState()

      // Check if bookmarked
      setIsBookmarked(bookmarks.some(b => b.url === url))

      // Scroll to top
      if (contentRef.current) {
        contentRef.current.scrollTop = 0
      }

    } catch (error) {
      console.error('Navigation failed:', error)
      setError(error instanceof Error ? error.message : 'Navigation failed')
    } finally {
      setLoading(false)
    }
  }, [parseUrl, getPublisher, showTableOfContents, historyIndex, bookmarks])

  // Extract title from markdown content
  const extractTitle = useCallback((content: string): string | null => {
    const titleMatch = content.match(/^#\s+(.+)$/m)
    return titleMatch ? titleMatch[1] : null
  }, [])

  // Update navigation state
  const updateNavigationState = useCallback(() => {
    setCanGoBack(historyIndex > 0)
    setCanGoForward(historyIndex < navigationHistory.length - 1)
  }, [historyIndex, navigationHistory.length])

  // Navigation handlers
  const goBack = useCallback(() => {
    if (canGoBack && historyIndex > 0) {
      const entry = navigationHistory[historyIndex - 1]
      setHistoryIndex(prev => prev - 1)
      navigateTo(entry.url, false)
    }
  }, [canGoBack, historyIndex, navigationHistory, navigateTo])

  const goForward = useCallback(() => {
    if (canGoForward && historyIndex < navigationHistory.length - 1) {
      const entry = navigationHistory[historyIndex + 1]
      setHistoryIndex(prev => prev + 1)
      navigateTo(entry.url, false)
    }
  }, [canGoForward, historyIndex, navigationHistory, navigateTo])

  const goHome = useCallback(() => {
    const { identity } = parseUrl(currentUrl)
    if (identity) {
      navigateTo(`${identity}/home.md`)
    }
  }, [currentUrl, parseUrl, navigateTo])

  const refresh = useCallback(() => {
    publishers.delete(parseUrl(currentUrl).identity)
    navigateTo(currentUrl, false)
  }, [currentUrl, parseUrl, navigateTo, publishers])

  // Address bar handler
  const handleAddressSubmit = useCallback((e: React.FormEvent) => {
    e.preventDefault()
    if (addressBarUrl !== currentUrl) {
      navigateTo(addressBarUrl)
    }
  }, [addressBarUrl, currentUrl, navigateTo])

  // Search functionality
  const performSearch = useCallback(async (query: string) => {
    if (!query.trim()) {
      setSearchResults([])
      return
    }

    setLoading(true)
    try {
      // Simplified search - in production would use full-text search index
      const results: SearchResult[] = []
      
      // Search through current identity's content
      const { identity } = parseUrl(currentUrl)
      const publisher = await getPublisher(identity)
      
      if (publisher) {
        // This is a mock implementation
        // In production, would have proper search indexing
        const mockResults = [
          {
            title: 'Home Page',
            url: `${identity}/home.md`,
            snippet: 'Welcome to our collaborative space...',
            identity,
            score: 0.9
          },
          {
            title: 'About Us',
            url: `${identity}/about.md`,
            snippet: 'Learn more about our mission...',
            identity,
            score: 0.7
          }
        ]
        
        results.push(...mockResults.filter(r => 
          r.title.toLowerCase().includes(query.toLowerCase()) ||
          r.snippet.toLowerCase().includes(query.toLowerCase())
        ))
      }
      
      setSearchResults(results.sort((a, b) => b.score - a.score))
    } catch (error) {
      console.error('Search failed:', error)
    } finally {
      setLoading(false)
    }
  }, [currentUrl, parseUrl, getPublisher])

  // Bookmark handlers
  const toggleBookmark = useCallback(() => {
    if (isBookmarked) {
      setBookmarks(prev => prev.filter(b => b.url !== currentUrl))
      setIsBookmarked(false)
    } else {
      const bookmark: Bookmark = {
        url: currentUrl,
        title,
        identity: parseUrl(currentUrl).identity,
        addedAt: Date.now()
      }
      setBookmarks(prev => [...prev, bookmark])
      setIsBookmarked(true)
    }
  }, [isBookmarked, currentUrl, title, parseUrl])

  // Handle link clicks in content
  const handleContentClick = useCallback((e: MouseEvent) => {
    const target = e.target as HTMLElement
    if (target.tagName === 'A') {
      e.preventDefault()
      const href = target.getAttribute('href')
      if (href) {
        if (href.startsWith('dht://')) {
          // DHT link to another identity
          const dhtUrl = href.replace('dht://', '')
          navigateTo(dhtUrl)
        } else if (!href.startsWith('http')) {
          // Relative link within same identity
          const { identity } = parseUrl(currentUrl)
          const newPath = href.startsWith('/') ? href.slice(1) : href
          navigateTo(`${identity}/${newPath}`)
        } else {
          // External link - open in new window
          window.open(href, '_blank')
        }
      }
    }
  }, [currentUrl, parseUrl, navigateTo])

  // Process content for rendering
  const processedContent = useMemo(() => {
    if (!content) return ''

    // Convert markdown to HTML (simplified)
    // In production would use a proper markdown processor
    let html = content
      // Headers
      .replace(/^### (.*$)/gim, '<h3 id="$1">$1</h3>')
      .replace(/^## (.*$)/gim, '<h2 id="$1">$1</h2>')
      .replace(/^# (.*$)/gim, '<h1 id="$1">$1</h1>')
      // Bold and italic
      .replace(/\*\*\*(.*)\*\*\*/gim, '<strong><em>$1</em></strong>')
      .replace(/\*\*(.*)\*\*/gim, '<strong>$1</strong>')
      .replace(/\*(.*)\*/gim, '<em>$1</em>')
      // Links
      .replace(/\[([^\]]+)\]\(([^)]+)\)/gim, '<a href="$2">$1</a>')
      // Line breaks
      .replace(/\n/gim, '<br>')

    return html
  }, [content])

  // Initialize
  useEffect(() => {
    navigateTo(initialUrl)
  }, [initialUrl, navigateTo])

  // Add content click handler
  useEffect(() => {
    const contentElement = contentRef.current
    if (contentElement) {
      contentElement.addEventListener('click', handleContentClick)
      return () => {
        contentElement.removeEventListener('click', handleContentClick)
      }
    }
  }, [handleContentClick])

  return (
    <Paper 
      className={className}
      sx={{
        height: isFullscreen ? '100vh' : '800px',
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
      {/* Navigation Bar */}
      <AppBar position="static" color="default" elevation={1}>
        <Toolbar variant="dense">
          {/* Navigation Controls */}
          <IconButton
            size="small"
            onClick={goBack}
            disabled={!canGoBack}
            sx={{ mr: 0.5 }}
          >
            <ArrowBack />
          </IconButton>
          
          <IconButton
            size="small"
            onClick={goForward}
            disabled={!canGoForward}
            sx={{ mr: 0.5 }}
          >
            <ArrowForward />
          </IconButton>
          
          <IconButton
            size="small"
            onClick={refresh}
            sx={{ mr: 0.5 }}
          >
            <Refresh />
          </IconButton>
          
          <IconButton
            size="small"
            onClick={goHome}
            sx={{ mr: 1 }}
          >
            <Home />
          </IconButton>

          {/* Address Bar */}
          <Box
            component="form"
            onSubmit={handleAddressSubmit}
            sx={{ flex: 1, mx: 2 }}
          >
            <TextField
              fullWidth
              size="small"
              value={addressBarUrl}
              onChange={(e) => setAddressBarUrl(e.target.value)}
              placeholder="Enter four-word-address/path.md"
              InputProps={{
                startAdornment: loading ? (
                  <InputAdornment position="start">
                    <CircularProgress size={16} />
                  </InputAdornment>
                ) : null,
              }}
              sx={{
                '& .MuiOutlinedInput-root': {
                  borderRadius: '20px'
                }
              }}
            />
          </Box>

          {/* Action Buttons */}
          {enableSearch && (
            <Tooltip title="Search">
              <IconButton
                size="small"
                onClick={() => setSearchOpen(true)}
                sx={{ mr: 0.5 }}
              >
                <Search />
              </IconButton>
            </Tooltip>
          )}

          {enableBookmarks && (
            <Tooltip title={isBookmarked ? "Remove Bookmark" : "Add Bookmark"}>
              <IconButton
                size="small"
                onClick={toggleBookmark}
                sx={{ mr: 0.5 }}
              >
                {isBookmarked ? <Bookmark /> : <BookmarkBorder />}
              </IconButton>
            </Tooltip>
          )}

          {showTableOfContents && tableOfContents.length > 0 && (
            <Tooltip title="Table of Contents">
              <IconButton
                size="small"
                onClick={() => setTocOpen(true)}
                sx={{ mr: 0.5 }}
              >
                <MenuIcon />
              </IconButton>
            </Tooltip>
          )}

          <Tooltip title="Menu">
            <IconButton
              size="small"
              onClick={(e) => setMenuAnchor(e.currentTarget)}
              sx={{ mr: 0.5 }}
            >
              <MenuIcon />
            </IconButton>
          </Tooltip>

          <Tooltip title={isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}>
            <IconButton
              size="small"
              onClick={() => setIsFullscreen(prev => !prev)}
            >
              {isFullscreen ? <FullscreenExit /> : <Fullscreen />}
            </IconButton>
          </Tooltip>
        </Toolbar>
      </AppBar>

      {/* Content Area */}
      <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        {/* Main Content */}
        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
          {/* Breadcrumbs */}
          <Box sx={{ p: 1, borderBottom: 1, borderColor: 'divider' }}>
            <Breadcrumbs separator="/">
              <Link
                component="button"
                variant="body2"
                onClick={() => {
                  const { identity } = parseUrl(currentUrl)
                  navigateTo(`${identity}/home.md`)
                }}
              >
                {parseUrl(currentUrl).identity}
              </Link>
              <Typography variant="body2" color="text.primary">
                {parseUrl(currentUrl).path}
              </Typography>
            </Breadcrumbs>
          </Box>

          {/* Error Display */}
          {error && (
            <Alert severity="error" sx={{ m: 2 }}>
              {error}
            </Alert>
          )}

          {/* Content */}
          <Box
            ref={contentRef}
            sx={{
              flex: 1,
              overflow: 'auto',
              p: 3,
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
              },
              '& a': {
                color: 'primary.main',
                textDecoration: 'none',
                '&:hover': {
                  textDecoration: 'underline'
                }
              }
            }}
            dangerouslySetInnerHTML={{ __html: processedContent }}
          />
        </Box>
      </Box>

      {/* Table of Contents Drawer */}
      <Drawer
        anchor="right"
        open={tocOpen}
        onClose={() => setTocOpen(false)}
        PaperProps={{ sx: { width: 300 } }}
      >
        <Box sx={{ p: 2 }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 2 }}>
            <Typography variant="h6">Contents</Typography>
            <IconButton size="small" onClick={() => setTocOpen(false)}>
              <Close />
            </IconButton>
          </Box>
          
          <List dense>
            {tableOfContents.map((entry, index) => (
              <ListItem
                key={index}
                button
                sx={{ pl: entry.level }}
                onClick={() => {
                  const element = document.getElementById(entry.id)
                  if (element) {
                    element.scrollIntoView({ behavior: 'smooth' })
                  }
                  setTocOpen(false)
                }}
              >
                <ListItemText primary={entry.text} />
              </ListItem>
            ))}
          </List>
        </Box>
      </Drawer>

      {/* Search Drawer */}
      <Drawer
        anchor="right"
        open={searchOpen}
        onClose={() => setSearchOpen(false)}
        PaperProps={{ sx: { width: 400 } }}
      >
        <Box sx={{ p: 2 }}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 2 }}>
            <Typography variant="h6">Search</Typography>
            <IconButton size="small" onClick={() => setSearchOpen(false)}>
              <Close />
            </IconButton>
          </Box>
          
          <TextField
            fullWidth
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value)
              performSearch(e.target.value)
            }}
            placeholder="Search content..."
            size="small"
            sx={{ mb: 2 }}
          />
          
          <List>
            {searchResults.map((result, index) => (
              <ListItem
                key={index}
                button
                onClick={() => {
                  navigateTo(result.url)
                  setSearchOpen(false)
                }}
              >
                <ListItemText
                  primary={result.title}
                  secondary={result.snippet}
                />
              </ListItem>
            ))}
          </List>
        </Box>
      </Drawer>

      {/* Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><Print /></ListItemIcon>
          Print
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><Download /></ListItemIcon>
          Save Page
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><Share /></ListItemIcon>
          Share
        </MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>
          <ListItemIcon><History /></ListItemIcon>
          History
        </MenuItem>
      </Menu>
    </Paper>
  )
}