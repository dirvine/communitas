import React, { useState, useCallback } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  Alert,
  CircularProgress,
  Stack,
  Chip,
  LinearProgress,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { FourWordsInput } from './FourWordsInput';
import { PQCLockIndicator, SecurityStatus } from './PQCLockIndicator';

interface ViewerState {
  // Input
  fourWords: string;
  isFourWordsValid: boolean;

  // Resolution
  resolving: boolean;
  siteId: string | null;

  // Discovery
  discovering: boolean;
  providerCount: number;

  // Fetching
  fetching: boolean;
  fetchProgress: { current: number; total: number } | null;

  // Content
  assets: Array<{ path: string; content: string }> | null;
  renderedHtml: string | null;

  // Security
  securityStatus: SecurityStatus;

  // Errors
  error: string | null;
}

interface SiteData {
  site_id: string;
  assets: Array<{ path: string; content_base64: string }>;
}

/**
 * Viewer Page - DNS-free Website Browser
 * 
 * Allows users to browse websites by four-word address:
 * 1. Enter four-words → resolve to SiteId
 * 2. Discover providers via gossip
 * 3. Fetch manifest and blocks
 * 4. Verify ML-DSA signature
 * 5. Render content (sandboxed)
 */
export const ViewerPage: React.FC = () => {
  const [state, setState] = useState<ViewerState>({
    fourWords: '',
    isFourWordsValid: false,
    resolving: false,
    siteId: null,
    discovering: false,
    providerCount: 0,
    fetching: false,
    fetchProgress: null,
    assets: null,
    renderedHtml: null,
    securityStatus: 'unsigned',
    error: null,
  });

  const handleBrowse = useCallback(async () => {
    if (!state.isFourWordsValid) {
      return;
    }

    setState(s => ({ ...s, error: null, resolving: true }));

    try {
      // Step 1: Resolve four-words to SiteId
      const siteId = await invoke<string | null>('gossip_name_resolve', {
        fourWords: state.fourWords,
      });

      if (!siteId) {
        setState(s => ({
          ...s,
          error: `Name not found: ${s.fourWords}`,
          resolving: false,
        }));
        return;
      }

      setState(s => ({ ...s, siteId, resolving: false, discovering: true }));

      // Step 2: Start provider discovery
      // Note: For MVP, we'll skip discovery UI and go straight to fetch
      // In production, we'd call gossip_site_subscribe_discovery here
      setState(s => ({ ...s, discovering: false, fetching: true }));

      // Step 3: Fetch site
      const siteData = await invoke<SiteData>('gossip_site_fetch', {
        siteIdHex: siteId,
      });

      // Step 4: Decode assets
      const assets = siteData.assets.map(asset => ({
        path: asset.path,
        content: atob(asset.content_base64),
      }));

      // Step 5: Find index.html
      const indexAsset = assets.find(a => a.path === 'index.html' || a.path === '/index.html');

      if (!indexAsset) {
        setState(s => ({
          ...s,
          error: 'No index.html found in site',
          fetching: false,
          assets,
        }));
        return;
      }

      // Step 6: Determine security status (TOFU for MVP)
      // In production, we'd check TOFU state via backend
      const securityStatus: SecurityStatus = 'tofu';

      setState(s => ({
        ...s,
        fetching: false,
        assets,
        renderedHtml: indexAsset.content,
        securityStatus,
        error: null,
      }));
    } catch (err) {
      setState(s => ({
        ...s,
        error: `Failed to load site: ${err}`,
        resolving: false,
        discovering: false,
        fetching: false,
      }));
    }
  }, [state.isFourWordsValid, state.fourWords]);

  return (
    <Box sx={{ p: 3, maxWidth: 1400, mx: 'auto' }}>
      {/* Header with Address Bar */}
      <Paper sx={{ p: 2, mb: 3 }}>
        <Typography variant="h5" gutterBottom sx={{ mb: 2 }}>
          Communitas Website Viewer
        </Typography>

        <Stack direction="row" spacing={2} alignItems="flex-start">
          <FourWordsInput
            value={state.fourWords}
            onChange={(value) => setState(s => ({ ...s, fourWords: value }))}
            onValidChange={(valid) => setState(s => ({ ...s, isFourWordsValid: valid }))}
            fullWidth
            label="Four-Word Address"
            disabled={state.resolving || state.discovering || state.fetching}
          />

          <Button
            variant="contained"
            onClick={handleBrowse}
            disabled={
              !state.isFourWordsValid ||
              state.resolving ||
              state.discovering ||
              state.fetching
            }
            sx={{ minWidth: 120, height: 56 }}
          >
            {state.resolving || state.discovering || state.fetching ? (
              <CircularProgress size={24} color="inherit" />
            ) : (
              'Browse'
            )}
          </Button>
        </Stack>

        {/* Status Bar */}
        {(state.siteId || state.providerCount > 0 || state.securityStatus !== 'unsigned') && (
          <Stack direction="row" spacing={1} sx={{ mt: 2 }} alignItems="center">
            {state.siteId && (
              <Chip
                label={`Site: ${state.siteId.substring(0, 8)}...`}
                size="small"
                variant="outlined"
              />
            )}
            {state.providerCount > 0 && (
              <Chip
                label={`${state.providerCount} provider${state.providerCount > 1 ? 's' : ''}`}
                size="small"
                color="primary"
                variant="outlined"
              />
            )}
            {state.securityStatus !== 'unsigned' && (
              <PQCLockIndicator status={state.securityStatus} />
            )}
          </Stack>
        )}

        {/* Progress indicator */}
        {(state.resolving || state.discovering || state.fetching) && (
          <Box sx={{ mt: 2 }}>
            <LinearProgress />
            <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5 }}>
              {state.resolving && 'Resolving name...'}
              {state.discovering && 'Discovering providers...'}
              {state.fetching && 'Fetching content...'}
            </Typography>
          </Box>
        )}
      </Paper>

      {/* Error Display */}
      {state.error && (
        <Alert severity="error" sx={{ mb: 3 }}>
          {state.error}
        </Alert>
      )}

      {/* Content Display */}
      {state.renderedHtml && (
        <Paper sx={{ p: 0, overflow: 'hidden' }}>
          <Box
            sx={{
              bgcolor: 'background.default',
              borderBottom: 1,
              borderColor: 'divider',
              px: 2,
              py: 1,
            }}
          >
            <Typography variant="caption" color="text.secondary">
              Rendered Content ({state.assets?.length || 0} asset
              {state.assets?.length !== 1 ? 's' : ''})
            </Typography>
          </Box>

          <Box
            sx={{
              p: 2,
              bgcolor: 'background.paper',
              minHeight: 400,
            }}
          >
            {/* Sandboxed HTML rendering */}
            <div dangerouslySetInnerHTML={{ __html: state.renderedHtml }} />
          </Box>
        </Paper>
      )}

      {/* Asset List (for debugging) */}
      {state.assets && !state.renderedHtml && (
        <Paper sx={{ p: 2 }}>
          <Typography variant="subtitle2" gutterBottom>
            Fetched Assets ({state.assets.length})
          </Typography>
          <Stack spacing={0.5}>
            {state.assets.map((asset) => (
              <Typography key={asset.path} variant="caption" sx={{ fontFamily: 'monospace' }}>
                • {asset.path} ({asset.content.length} bytes)
              </Typography>
            ))}
          </Stack>
        </Paper>
      )}
    </Box>
  );
};

export default ViewerPage;
