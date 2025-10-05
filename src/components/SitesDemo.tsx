// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

/**
 * Saorsa Sites Demo Component
 *
 * Simple UI for demonstrating DNS-free website publishing
 * via the rendezvous protocol.
 */

import React, { useState } from 'react';
import {
  Box,
  Button,
  Card,
  CardContent,
  TextField,
  Typography,
  Alert,
  CircularProgress,
  List,
  ListItem,
  ListItemText,
  Divider,
} from '@mui/material';
import { sitesService, SitesService, AssetData } from '../services/SitesService';

export const SitesDemo: React.FC = () => {
  const [htmlContent, setHtmlContent] = useState('<html><body><h1>Hello from Saorsa Sites!</h1></body></html>');
  const [cssContent, setCssContent] = useState('body { font-family: sans-serif; }');
  const [publishedSiteId, setPublishedSiteId] = useState('');
  const [fetchSiteId, setFetchSiteId] = useState('');
  const [fetchedSite, setFetchedSite] = useState<{ path: string; content: string }[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  const handlePublish = async () => {
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const assets: AssetData[] = [
        SitesService.fromString('index.html', htmlContent),
        SitesService.fromString('style.css', cssContent),
      ];

      const siteId = await sitesService.publish(assets);
      setPublishedSiteId(siteId);
      setSuccess(`Site published! ID: ${siteId.substring(0, 16)}...`);
    } catch (err) {
      setError(`Publish failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleFetch = async () => {
    if (!fetchSiteId) {
      setError('Please enter a site ID');
      return;
    }

    setLoading(true);
    setError('');
    setSuccess('');
    setFetchedSite([]);

    try {
      const site = await sitesService.fetch(fetchSiteId);
      const decoded = site.assets.map(asset => ({
        path: asset.path,
        content: atob(asset.content_base64),
      }));
      setFetchedSite(decoded);
      setSuccess(`Fetched ${decoded.length} assets`);
    } catch (err) {
      setError(`Fetch failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleList = async () => {
    setLoading(true);
    setError('');
    setSuccess('');

    try {
      const sites = await sitesService.list();
      if (sites.length > 0) {
        setSuccess(`Found ${sites.length} site(s). First: ${sites[0].substring(0, 16)}...`);
        setFetchSiteId(sites[0]);
      } else {
        setSuccess('No published sites found');
      }
    } catch (err) {
      setError(`List failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box sx={{ p: 3, maxWidth: 1200, mx: 'auto' }}>
      <Typography variant="h4" gutterBottom>
        Saorsa Sites Demo
      </Typography>
      <Typography variant="body2" color="text.secondary" paragraph>
        DNS-free website publishing via rendezvous protocol
      </Typography>

      {/* Publish Section */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Publish Site
          </Typography>

          <TextField
            label="index.html"
            multiline
            rows={4}
            fullWidth
            value={htmlContent}
            onChange={(e) => setHtmlContent(e.target.value)}
            sx={{ mb: 2 }}
            variant="outlined"
          />

          <TextField
            label="style.css"
            multiline
            rows={2}
            fullWidth
            value={cssContent}
            onChange={(e) => setCssContent(e.target.value)}
            sx={{ mb: 2 }}
            variant="outlined"
          />

          <Button
            variant="contained"
            onClick={handlePublish}
            disabled={loading}
            fullWidth
          >
            {loading ? <CircularProgress size={24} /> : 'Publish Site'}
          </Button>

          {publishedSiteId && (
            <Alert severity="info" sx={{ mt: 2 }}>
              <Typography variant="caption" sx={{ fontFamily: 'monospace' }}>
                Site ID: {publishedSiteId}
              </Typography>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Fetch Section */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Fetch Site
          </Typography>

          <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
            <TextField
              label="Site ID (hex)"
              fullWidth
              value={fetchSiteId}
              onChange={(e) => setFetchSiteId(e.target.value)}
              variant="outlined"
              placeholder="Enter site ID or click List to auto-fill"
            />
            <Button variant="outlined" onClick={handleList} disabled={loading}>
              List
            </Button>
          </Box>

          <Button
            variant="contained"
            onClick={handleFetch}
            disabled={loading || !fetchSiteId}
            fullWidth
          >
            {loading ? <CircularProgress size={24} /> : 'Fetch Site'}
          </Button>

          {fetchedSite.length > 0 && (
            <Box sx={{ mt: 2 }}>
              <Typography variant="subtitle2" gutterBottom>
                Fetched Assets:
              </Typography>
              <List>
                {fetchedSite.map((asset, idx) => (
                  <React.Fragment key={idx}>
                    <ListItem>
                      <ListItemText
                        primary={asset.path}
                        secondary={
                          <Typography
                            variant="caption"
                            sx={{
                              fontFamily: 'monospace',
                              whiteSpace: 'pre-wrap',
                              display: 'block',
                              maxHeight: 200,
                              overflow: 'auto',
                              bgcolor: 'grey.100',
                              p: 1,
                              borderRadius: 1,
                            }}
                          >
                            {asset.content}
                          </Typography>
                        }
                      />
                    </ListItem>
                    {idx < fetchedSite.length - 1 && <Divider />}
                  </React.Fragment>
                ))}
              </List>
            </Box>
          )}
        </CardContent>
      </Card>

      {/* Status Messages */}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {error}
        </Alert>
      )}
      {success && (
        <Alert severity="success" sx={{ mb: 2 }}>
          {success}
        </Alert>
      )}
    </Box>
  );
};
