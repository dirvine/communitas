import { Close as CloseIcon, Download as DownloadIcon } from '@mui/icons-material';
import {
    Alert, Box, Button, Dialog, DialogActions, DialogContent, DialogTitle, Stack, Typography
} from '@mui/material';
import { QRCodeSVG } from 'qrcode.react';
import React from 'react';

interface QRCodeExportDialogProps {
  open: boolean;
  onClose: () => void;
  fourWords: string;
  displayName: string;
}

interface QRCodeData {
  type: 'communitas-identity';
  version: '1.0';
  fourWords: string;
  displayName: string;
}

export const QRCodeExportDialog: React.FC<QRCodeExportDialogProps> = ({
  open,
  onClose,
  fourWords,
  displayName,
}) => {
  const qrData: QRCodeData = {
    type: 'communitas-identity',
    version: '1.0',
    fourWords,
    displayName,
  };
  const exportData = JSON.stringify(qrData);
  const handleDownloadQR = () => {
    const canvas = document.createElement('canvas');
    const svg = document.querySelector('#qr-export-svg') as HTMLElement;
    if (!svg) return;

    // Convert SVG to PNG and download
    const svgData = new XMLSerializer().serializeToString(svg);
    const img = new Image();
    img.onload = () => {
      canvas.width = img.width;
      canvas.height = img.height;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      ctx.fillStyle = 'white';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0);

      canvas.toBlob((blob) => {
        if (!blob) return;
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.download = `communitas-${fourWords}.png`;
        link.href = url;
        link.click();
        URL.revokeObjectURL(url);
      });
    };
    img.src = 'data:image/svg+xml;base64,' + btoa(svgData);
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Typography variant="h6" fontWeight={600}>
            Export Identity QR Code
          </Typography>
          <Button
            startIcon={<CloseIcon />}
            onClick={onClose}
            size="small"
          >
            Close
          </Button>
        </Box>
      </DialogTitle>

      <DialogContent>
        <Stack spacing={3}>
          <Alert severity="info">
            <Typography variant="body2">
              Scan this QR code on another device to import this identity.
              You'll still need to enter your password to complete the setup.
            </Typography>
          </Alert>

          <Box
            sx={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: 2,
            }}
          >
            <Typography variant="body2" color="text.secondary" align="center">
              Identity: <strong>{displayName}</strong>
            </Typography>
            <Typography
              variant="caption"
              color="text.secondary"
              sx={{ fontFamily: 'monospace' }}
              align="center"
            >
              {fourWords}
            </Typography>

            <Box
              sx={{
                p: 3,
                bgcolor: 'white',
                borderRadius: 2,
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
              }}
            >
              <QRCodeSVG
                id="qr-export-svg"
                value={exportData}
                size={256}
                level="H"
                includeMargin={true}
              />
            </Box>

            <Alert severity="info" sx={{ width: '100%' }}>
              <Typography variant="caption">
                This QR code only contains your four-word address and display name.
                Your password is never included for security.
              </Typography>
            </Alert>
          </Box>
        </Stack>
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 3 }}>
        <Button
          variant="outlined"
          startIcon={<DownloadIcon />}
          onClick={handleDownloadQR}
        >
          Download QR Code
        </Button>
        <Button variant="contained" onClick={onClose}>
          Done
        </Button>
      </DialogActions>
    </Dialog>
  );
};
