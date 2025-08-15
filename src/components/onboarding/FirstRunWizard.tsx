import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import {
  Box,
  Button,
  Dialog,
  DialogContent,
  DialogTitle,
  Step,
  StepContent,
  StepLabel,
  Stepper,
  Typography,
  Alert,
  CircularProgress,
} from '@mui/material'
import IdentitySetup from '../identity/IdentitySetup'

interface FirstRunWizardProps {
  open: boolean
  onClose: () => void
}

const FirstRunWizard: React.FC<FirstRunWizardProps> = ({ open, onClose }) => {
  const [activeStep, setActiveStep] = useState(0)
  const [networkReady, setNetworkReady] = useState<boolean>(false)
  const [checkingNetwork, setCheckingNetwork] = useState<boolean>(false)
  const [identityCreated, setIdentityCreated] = useState<boolean>(false)
  const [error, setError] = useState<string | null>(null)

  const steps = [
    { label: 'Create Identity', description: 'Generate your four‑word identity and keys' },
    { label: 'Initialize Network', description: 'Start P2P node and verify connectivity' },
    { label: 'Secure Storage', description: 'Verify OS keychain and encryption keys' },
  ]

  useEffect(() => {
    if (!open) return
    // Pre-check secure storage availability
    ;(async () => {
      try {
        await invoke('get_secure_storage_info')
      } catch (e) {
        // Surface later in step 3
      }
    })()
  }, [open])

  const startNetwork = async () => {
    setCheckingNetwork(true)
    setError(null)
    try {
      await invoke<string>('initialize_p2p_node')
      // poll once for health
      const health = await invoke<any>('get_network_health')
      setNetworkReady((health as any)?.status === 'connected' || (health as any)?.peer_count > 0)
    } catch (e: any) {
      setError(typeof e === 'string' ? e : 'Failed to initialize network')
      setNetworkReady(false)
    } finally {
      setCheckingNetwork(false)
    }
  }

  const verifySecureStorage = async () => {
    try {
      const info = await invoke<any>('get_secure_storage_info')
      if (!info || !info.available) {
        setError('Secure storage not available; features may be limited')
      }
    } catch (e) {
      setError('Failed to query secure storage')
    }
  }

  const onIdentityCreated = () => {
    setIdentityCreated(true)
    setActiveStep(1)
    // Mark onboarded immediately to avoid re-opening wizard on refresh
    localStorage.setItem('communitas-onboarded', 'true')
  }

  const onContinue = async () => {
    if (activeStep === 1) {
      await startNetwork()
      setActiveStep(2)
    } else if (activeStep === 2) {
      await verifySecureStorage()
      onClose()
    }
  }

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>Welcome to Communitas</DialogTitle>
      <DialogContent>
        <Stepper activeStep={activeStep} orientation="vertical">
          {steps.map((s, idx) => (
            <Step key={s.label}>
              <StepLabel>{s.label}</StepLabel>
              <StepContent>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  {s.description}
                </Typography>
                {idx === 0 && (
                  <IdentitySetup
                    open={true}
                    onClose={() => {}}
                    onIdentityCreated={() => onIdentityCreated()}
                  />
                )}
                {idx === 1 && (
                  <Box sx={{ mt: 2 }}>
                    {checkingNetwork ? (
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                        <CircularProgress size={24} />
                        <Typography>Connecting to P2P network…</Typography>
                      </Box>
                    ) : (
                      <>
                        {!networkReady && (
                          <Alert severity="info" sx={{ mb: 2 }}>
                            We will start your node and connect to bootstrap peers.
                          </Alert>
                        )}
                        {error && (
                          <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>
                        )}
                        <Button variant="contained" onClick={startNetwork}>Initialize Network</Button>
                      </>
                    )}
                  </Box>
                )}
                {idx === 2 && (
                  <Box sx={{ mt: 2 }}>
                    <Typography gutterBottom>
                      We will verify secure storage availability and finish setup.
                    </Typography>
                    {error && (
                      <Alert severity="warning" sx={{ mb: 2 }}>{error}</Alert>
                    )}
                    <Button variant="contained" onClick={onClose}>Finish</Button>
                  </Box>
                )}
              </StepContent>
            </Step>
          ))}
        </Stepper>
      </DialogContent>
    </Dialog>
  )
}

export default FirstRunWizard
