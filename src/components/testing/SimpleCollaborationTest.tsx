import React, { useState } from 'react';
import { Box, Button, Typography, Paper, TextField } from '@mui/material';
import { YjsMarkdownEditor } from '../../services/storage/yjsCollaboration';

export const SimpleCollaborationTest: React.FC = () => {
  const [editor, setEditor] = useState<YjsMarkdownEditor | null>(null);
  const [content, setContent] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const [userId, setUserId] = useState('user_' + Math.random().toString(36).substr(2, 9));

  const initializeEditor = async () => {
    try {
      const roomId = 'test-room-' + Date.now();
      const yjsEditor = new YjsMarkdownEditor(userId, roomId);

      // Set user info
      yjsEditor.setUserInfo({
        name: userId,
        color: '#4CAF50'
      });

      // Listen for content changes
      yjsEditor.onContentChange((newContent) => {
        setContent(newContent);
      });

      // Connect to the room
      await yjsEditor.connect();
      setIsConnected(true);
      setEditor(yjsEditor);

      console.log('✅ Collaborative editor initialized successfully!');
      console.log('📝 Room ID:', roomId);
      console.log('👤 User ID:', userId);

    } catch (error) {
      console.error('❌ Failed to initialize editor:', error);
    }
  };

  const handleContentChange = (event: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newContent = event.target.value;
    setContent(newContent);

    if (editor) {
      // Replace the entire content (simplified for testing)
      editor.replaceText(editor.getContent(), newContent);
    }
  };

  const cleanup = async () => {
    if (editor) {
      await editor.destroy();
      setEditor(null);
      setIsConnected(false);
      setContent('');
      console.log('🧹 Editor cleaned up');
    }
  };

  return (
    <Box sx={{ p: 3, maxWidth: '800px', mx: 'auto' }}>
      <Typography variant="h4" gutterBottom>
        Simple Collaboration Test
      </Typography>

      <Typography variant="body1" sx={{ mb: 3 }}>
        Test basic collaborative editing functionality with a simple text area.
      </Typography>

      {/* Control Panel */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2 }}>
          <TextField
            label="User ID"
            value={userId}
            onChange={(e) => setUserId(e.target.value)}
            size="small"
            sx={{ minWidth: 200 }}
          />
          <Button
            variant="contained"
            onClick={initializeEditor}
            disabled={isConnected}
          >
            {isConnected ? 'Connected' : 'Initialize Editor'}
          </Button>
          <Button
            variant="outlined"
            onClick={cleanup}
            disabled={!isConnected}
          >
            Cleanup
          </Button>
        </Box>

        <Typography variant="body2" color="text.secondary">
          Status: {isConnected ? '🟢 Connected' : '🔴 Disconnected'} |
          Content Length: {content.length} characters
        </Typography>
      </Paper>

      {/* Editor */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Collaborative Text Editor
        </Typography>

        <TextField
          multiline
          rows={10}
          fullWidth
          value={content}
          onChange={handleContentChange}
          placeholder="Start typing to test collaborative editing..."
          disabled={!isConnected}
          sx={{
            '& .MuiInputBase-root': {
              fontFamily: 'monospace',
            }
          }}
        />
      </Paper>

      {/* Instructions */}
      <Paper sx={{ p: 2 }}>
        <Typography variant="h6" gutterBottom>
          🧪 Testing Instructions
        </Typography>
        <Typography variant="body2" component="div">
          <ol>
            <li>Click "Initialize Editor" to set up the collaborative session</li>
            <li>Start typing in the text area above</li>
            <li>Open this page in another browser tab</li>
            <li>Use the same User ID in both tabs</li>
            <li>Watch changes sync between tabs in real-time</li>
            <li>Check the browser console for connection logs</li>
          </ol>
        </Typography>

        <Typography variant="body2" sx={{ mt: 2, fontStyle: 'italic' }}>
          💡 Tip: Open browser developer tools (F12) to see real-time collaboration events in the console.
        </Typography>
      </Paper>
    </Box>
  );
};