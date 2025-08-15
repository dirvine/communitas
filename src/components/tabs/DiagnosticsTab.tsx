import React, { useState } from 'react'
import {
  Box,
  Typography,
  Card,
  CardContent,
  Button,
  TextField,
  FormControlLabel,
  Switch,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
} from '@mui/material'
import { PlayArrow, Stop, Clear } from '@mui/icons-material'

const DiagnosticsTab: React.FC = () => {
  const [packetCapture, setPacketCapture] = useState(false)
  const [filter, setFilter] = useState('')
  
  const packets = [
    {
      id: '1',
      timestamp: '16:45:23.123',
      protocol: 'QUIC',
      source: '192.168.1.100:9001',
      destination: '203.0.113.5:8888',
      size: 1200,
      info: 'Initial packet',
    },
    {
      id: '2',
      timestamp: '16:45:23.156',
      protocol: 'QUIC',
      source: '203.0.113.5:8888',
      destination: '192.168.1.100:9001',
      size: 800,
      info: 'Version negotiation',
    },
    {
      id: '3',
      timestamp: '16:45:23.189',
      protocol: 'DHT',
      source: '192.168.1.100:9001',
      destination: '198.51.100.42:9002',
      size: 456,
      info: 'FIND_NODE query',
    },
  ]

  const filteredPackets = packets.filter(packet =>
    !filter || packet.protocol.toLowerCase().includes(filter.toLowerCase()) ||
    packet.info.toLowerCase().includes(filter.toLowerCase())
  )

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Advanced Diagnostics
      </Typography>
      
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Packet Capture
          </Typography>
          
          <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2 }}>
            <FormControlLabel
              control={
                <Switch
                  checked={packetCapture}
                  onChange={(e) => setPacketCapture(e.target.checked)}
                />
              }
              label="Enable Packet Capture"
            />
            
            <Button
              variant="contained"
              startIcon={packetCapture ? <Stop /> : <PlayArrow />}
              onClick={() => setPacketCapture(!packetCapture)}
              color={packetCapture ? 'error' : 'primary'}
            >
              {packetCapture ? 'Stop' : 'Start'} Capture
            </Button>
            
            <Button
              variant="outlined"
              startIcon={<Clear />}
              disabled={packetCapture}
            >
              Clear
            </Button>
          </Box>
          
          <TextField
            fullWidth
            label="Filter packets"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter by protocol or info..."
            sx={{ mb: 2 }}
          />
          
          <Typography variant="body2" color="textSecondary">
            Status: {packetCapture ? 
              <Chip label="Capturing" color="success" size="small" /> : 
              <Chip label="Stopped" color="default" size="small" />
            }
          </Typography>
        </CardContent>
      </Card>
      
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Captured Packets ({filteredPackets.length})
          </Typography>
          
          <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
            <Table stickyHeader size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Time</TableCell>
                  <TableCell>Protocol</TableCell>
                  <TableCell>Source</TableCell>
                  <TableCell>Destination</TableCell>
                  <TableCell>Size</TableCell>
                  <TableCell>Info</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {filteredPackets.map((packet) => (
                  <TableRow key={packet.id} hover>
                    <TableCell>{packet.timestamp}</TableCell>
                    <TableCell>
                      <Chip label={packet.protocol} size="small" />
                    </TableCell>
                    <TableCell>{packet.source}</TableCell>
                    <TableCell>{packet.destination}</TableCell>
                    <TableCell>{packet.size}B</TableCell>
                    <TableCell>{packet.info}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </CardContent>
      </Card>
    </Box>
  )
}

export default DiagnosticsTab
