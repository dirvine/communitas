// Server routes setup with WebSocket integration
import { createServer } from 'http';
import express from 'express';
import { CommunicationWebSocketServer } from './websocket.js';

const app = express();
app.use(express.json());

// Health check endpoint
app.get('/api/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

// WebSocket stats endpoint  
let wsServer: CommunicationWebSocketServer;

app.get('/api/ws-stats', (req, res) => {
  if (wsServer) {
    res.json(wsServer.getStats());
  } else {
    res.json({ error: 'WebSocket server not initialized' });
  }
});

// Create HTTP server
const httpServer = createServer(app);

// Initialize WebSocket server
wsServer = new CommunicationWebSocketServer(httpServer);

const PORT = process.env.PORT || 3001;

httpServer.listen(PORT, () => {
  console.log(`🚀 Server running on port ${PORT}`);
  console.log(`🔌 WebSocket server available at ws://localhost:${PORT}/ws`);
});

export { httpServer, wsServer };