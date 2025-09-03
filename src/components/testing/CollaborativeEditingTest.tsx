import React, { useState } from 'react';
import { Box, Button, Typography, Paper, Grid, Chip } from '@mui/material';
import { CollaborativeMarkdownEditor } from '../editor/CollaborativeMarkdownEditor';
import { NetworkIdentity } from '../../types/collaboration';

export const CollaborativeEditingTest: React.FC = () => {
  const [testMode, setTestMode] = useState<'single' | 'multi'>('single');
  const [userCount, setUserCount] = useState(1);

  // Create test users
  const testUsers: NetworkIdentity[] = [
    {
      fourWords: 'ocean-forest-moon-star',
      publicKey: 'test-key-1',
      dhtAddress: 'test-dht-1'
    },
    {
      fourWords: 'river-stone-cloud-dream',
      publicKey: 'test-key-2',
      dhtAddress: 'test-dht-2'
    },
    {
      fourWords: 'mountain-valley-wind-rain',
      publicKey: 'test-key-3',
      dhtAddress: 'test-dht-3'
    }
  ];

  const sampleContent = `# Collaborative Markdown Editor Test

This is a test document for the collaborative editing functionality.

## Features to Test

- **Real-time editing**: Type and see changes sync instantly
- **User awareness**: See other users' cursors and selections
- **Conflict resolution**: Simultaneous edits are merged automatically
- **Markdown support**: Full markdown syntax with preview

## Test Instructions

1. Open this editor in multiple browser tabs/windows
2. Start typing in different sections
3. Notice how changes sync between instances
4. Try selecting text to see cursor indicators
5. Test the preview mode and split view

## Code Example

\`\`\`javascript
// This is a code block
function testCollaboration() {
  console.log('Testing collaborative editing!');
  return 'Success!';
}
\`\`\`

## Checklist

- [ ] Real-time synchronization working
- [ ] User cursors visible
- [ ] Conflict resolution functional
- [ ] Markdown rendering correct
- [ ] Performance acceptable

---

*Last updated: ${new Date().toLocaleString()}*
`;

  const handleSave = async (content: string) => {
    console.log('Content saved:', content.length, 'characters');
    // In a real implementation, this would save to storage
  };

  const handlePublish = async () => {
    console.log('Document published');
    // In a real implementation, this would publish the document
  };

  return (
    <Box sx={{ p: 3, maxWidth: '1200px', mx: 'auto' }}>
      <Typography variant="h4" gutterBottom>
        Collaborative Editing Test Suite
      </Typography>

      <Typography variant="body1" sx={{ mb: 3, color: 'text.secondary' }}>
        Test the real-time collaborative editing functionality with multiple users and instances.
      </Typography>

      {/* Test Controls */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Test Configuration
        </Typography>

        <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2 }}>
          <Button
            variant={testMode === 'single' ? 'contained' : 'outlined'}
            onClick={() => setTestMode('single')}
          >
            Single User Test
          </Button>
          <Button
            variant={testMode === 'multi' ? 'contained' : 'outlined'}
            onClick={() => setTestMode('multi')}
          >
            Multi-User Test
          </Button>
        </Box>

        {testMode === 'multi' && (
          <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
            <Typography variant="body2">Simulated Users:</Typography>
            {[1, 2, 3].map(count => (
              <Chip
                key={count}
                label={`${count} user${count > 1 ? 's' : ''}`}
                onClick={() => setUserCount(count)}
                color={userCount === count ? 'primary' : 'default'}
                variant={userCount === count ? 'filled' : 'outlined'}
                size="small"
              />
            ))}
          </Box>
        )}
      </Paper>

      {/* Test Instructions */}
      <Paper sx={{ p: 2, mb: 3, bgcolor: 'info.main', color: 'info.contrastText' }}>
        <Typography variant="h6" gutterBottom>
          🧪 Testing Instructions
        </Typography>
        <Typography variant="body2" component="div">
          <ol>
            <li>Open this page in {testMode === 'multi' ? userCount : 1} browser tab{testMode === 'multi' && userCount > 1 ? 's' : ''}</li>
            <li>Start typing in the editor below</li>
            <li>Watch for real-time synchronization between tabs</li>
            <li>Try selecting text to see cursor indicators</li>
            <li>Test the preview and split view modes</li>
            <li>Verify that all changes are preserved</li>
          </ol>
        </Typography>
      </Paper>

      {/* Collaborative Editor */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Collaborative Markdown Editor
        </Typography>

        <Box sx={{ height: '600px', border: '1px solid', borderColor: 'divider', borderRadius: 1 }}>
          <CollaborativeMarkdownEditor
            filePath="test-collaboration.md"
            entityId="test-entity-123"
            currentUser={testUsers[0]}
            initialContent={sampleContent}
            onSave={handleSave}
            onPublish={handlePublish}
            showPreview={true}
            showCollaborators={true}
            enableVersionHistory={true}
            theme="auto"
          />
        </Box>
      </Paper>

      {/* Status Panel */}
      <Grid container spacing={2}>
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 2 }}>
            <Typography variant="h6" gutterBottom>
              ✅ Features Implemented
            </Typography>
            <Box component="ul" sx={{ pl: 2, m: 0 }}>
              <li>Real-time collaborative editing</li>
              <li>Yjs CRDT synchronization</li>
              <li>Monaco Editor integration</li>
              <li>User awareness and cursors</li>
              <li>Markdown preview and syntax highlighting</li>
              <li>Version history and snapshots</li>
              <li>Offline persistence with IndexedDB</li>
              <li>Conflict resolution (Operational Transformation)</li>
            </Box>
          </Paper>
        </Grid>

        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 2 }}>
            <Typography variant="h6" gutterBottom>
              🔧 Technical Details
            </Typography>
            <Box sx={{ fontFamily: 'monospace', fontSize: '0.875rem' }}>
              <div>Yjs Version: 13.6.21</div>
              <div>WebRTC Provider: y-webrtc</div>
              <div>Editor: Monaco Editor</div>
              <div>Storage: IndexedDB</div>
              <div>Signaling: LocalStorage (Demo)</div>
              <div>Conflict Resolution: OT</div>
            </Box>
          </Paper>
        </Grid>
      </Grid>

      {/* Test Results */}
      <Paper sx={{ p: 2, mt: 3 }}>
        <Typography variant="h6" gutterBottom>
          📊 Test Results
        </Typography>
        <Typography variant="body2" color="text.secondary">
          Check the browser console for detailed logging of collaborative events.
          Open multiple tabs to test real-time synchronization.
        </Typography>
      </Paper>
    </Box>
  );
};