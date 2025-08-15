import React, { useState } from 'react';
import { Box, Card, CardContent, Typography, TextField, Button, Alert } from '@mui/material';
import { invoke } from '@tauri-apps/api/tauri';

interface DhtOperationsProps {
  className?: string;
}

export const DhtOperations: React.FC<DhtOperationsProps> = ({ className }) => {
  const [content, setContent] = useState('');
  const [result, setResult] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleStore = async () => {
    if (\!content.trim()) return;
    
    setLoading(true);
    setError(null);
    try {
      const key = await invoke<string>('dht_store_content', { content });
      setResult('Stored with key: ' + key);
    } catch (err) {
      setError('Error: ' + err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box className={className}>
      <Card>
        <CardContent>
          <Typography variant="h6" mb={2}>DHT Operations</Typography>
          <TextField
            fullWidth
            label="Content to Store"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            disabled={loading}
            sx={{ mb: 2 }}
          />
          <Button
            variant="contained"
            onClick={handleStore}
            disabled={loading || \!content.trim()}
          >
            {loading ? 'Storing...' : 'Store Content'}
          </Button>
          {error && <Alert severity="error" sx={{ mt: 2 }}>{error}</Alert>}
          {result && <Alert severity="success" sx={{ mt: 2 }}>{result}</Alert>}
        </CardContent>
      </Card>
    </Box>
  );
};

export default DhtOperations;
EOFILE < /dev/null