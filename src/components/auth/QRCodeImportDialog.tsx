import {
    Close as CloseIcon,
    QrCodeScanner as QrCodeScannerIcon,
    Upload as UploadIcon
} from '@mui/icons-material';
import {
    Alert, Box, Button, CircularProgress, Dialog, DialogActions, DialogContent, DialogTitle, Stack, Typography
} from '@mui/material';
import { Html5Qrcode } from 'html5-qrcode';
import React, { useEffect, useRef, useState } from 'react';

interface QRCodeImportDialogProps {
  open: boolean;
  onClose: () => void;
  onImport: (data: string) => Promise<void>;
}

export const QRCodeImportDialog: React.FC<QRCodeImportDialogProps> = ({
  open,
  onClose,
  onImport,
}) => {
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [importing, setImporting] = useState(false);
  const scannerRef = useRef<Html5Qrcode | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    return () => {
      // Cleanup scanner on unmount
      if (scannerRef.current) {
        scannerRef.current.stop().catch(console.error);
      }
    };
  }, []);

  const startScanning = async () => {
    try {
      setError(null);
      setScanning(true);

      const scanner = new Html5Qrcode('qr-reader');
      scannerRef.current = scanner;

      await scanner.start(
        { facingMode: 'environment' },
        {
          fps: 10,
          qrbox: { width: 250, height: 250 },
        },
        async (decodedText) => {
          // QR code scanned successfully
          await handleQRCodeScanned(decodedText);
        },
        (errorMessage) => {
          // Scanning error (ignore, happens often)
          console.debug('QR scan error:', errorMessage);
        }
      );
    } catch (err) {
      console.error('Failed to start scanner:', err);
      setError('Failed to access camera. Please check permissions or upload a QR code image.');
      setScanning(false);
    }
  };

  const stopScanning = async () => {
    if (scannerRef.current) {
      try {
        await scannerRef.current.stop();
      } catch (err) {
        console.error('Failed to stop scanner:', err);
      }
      scannerRef.current = null;
    }
    setScanning(false);
  };

  const handleQRCodeScanned = async (data: string) => {
    try {
      setImporting(true);
      setError(null);

      await stopScanning();
      await onImport(data);

      setSuccess(true);
      setTimeout(() => {
        onClose();
      }, 2000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to import identity');
    } finally {
      setImporting(false);
    }
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      setError(null);
      setImporting(true);

      const scanner = new Html5Qrcode('qr-reader-file');
      const result = await scanner.scanFile(file, true);

      await handleQRCodeScanned(result);
    } catch (err) {
      setError('Failed to read QR code from image. Please try again.');
      console.error('QR scan from file error:', err);
    } finally {
      setImporting(false);
    }
  };

  const handleClose = async () => {
    await stopScanning();
    setError(null);
    setSuccess(false);
    onClose();
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Typography variant="h6" fontWeight={600}>
            Import Identity from QR Code
          </Typography>
          <Button
            startIcon={<CloseIcon />}
            onClick={handleClose}
            size="small"
            disabled={importing}
          >
            Close
          </Button>
        </Box>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={3}>
          {error && (
            <Alert severity="error" onClose={() => setError(null)}>
              {error}
            </Alert>
          )}

          {success && (
            <Alert severity="success">
              Identity imported successfully! You can now sign in.
            </Alert>
          )}

          {!scanning && !success && (
            <>
              <Alert severity="info">
                <Typography variant="body2">
                  Scan a QR code from another device or upload a saved QR code image
                  to import an identity.
                </Typography>
              </Alert>

              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                <Button
                  variant="contained"
                  startIcon={<QrCodeScannerIcon />}
                  onClick={startScanning}
                  disabled={importing}
                  fullWidth
                  size="large"
                >
                  Scan QR Code with Camera
                </Button>

                <Button
                  variant="outlined"
                  startIcon={<UploadIcon />}
                  onClick={() => fileInputRef.current?.click()}
                  disabled={importing}
                  fullWidth
                  size="large"
                >
                  Upload QR Code Image
                </Button>

                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  style={{ display: 'none' }}
                  onChange={handleFileUpload}
                />
              </Box>
            </>
          )}

          {scanning && (
            <Box
              sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: 2,
              }}
            >
              <Typography variant="body2" color="text.secondary">
                Position the QR code within the frame
              </Typography>
              <Box
                id="qr-reader"
                sx={{
                  width: '100%',
                  '& video': {
                    borderRadius: 2,
                  },
                }}
              />
              <Button
                variant="outlined"
                onClick={stopScanning}
                disabled={importing}
              >
                Stop Scanning
              </Button>
            </Box>
          )}

          {importing && (
            <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 2 }}>
              <CircularProgress size={48} />
              <Typography variant="body2" color="text.secondary">
                Importing identity...
              </Typography>
            </Box>
          )}

          {/* Hidden div for file upload scanning */}
          <div id="qr-reader-file" style={{ display: 'none' }} />
        </Stack>
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button onClick={handleClose} disabled={importing}>
          Cancel
        </Button>
      </DialogActions>
    </Dialog>
  );
};
