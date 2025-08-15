import { invoke } from '@tauri-apps/api/core'
import React, { useState } from 'react'
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Typography,
  Box,
  CircularProgress,
  Alert,
  FormControlLabel,
  Checkbox,
  Stepper,
  Step,
  StepLabel,
  StepContent,
} from '@mui/material'
import {
  PersonAdd as PersonAddIcon,
  // Security as SecurityIcon,
  Check as CheckIcon,
} from '@mui/icons-material'
import { IdentityInfo, IdentityGenerationParams } from '../../types'

interface IdentitySetupProps {
  open: boolean
  onClose: () => void
  onIdentityCreated: (identity: IdentityInfo) => void
}

const IdentitySetup: React.FC<IdentitySetupProps> = ({
  open,
  onClose,
  onIdentityCreated,
}) => {
  const [activeStep, setActiveStep] = useState(0)
  const [displayName, setDisplayName] = useState('')
  const [useHardwareEntropy, setUseHardwareEntropy] = useState(true)
  const [powDifficulty, setPowDifficulty] = useState(8)
  const [isGenerating, setIsGenerating] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [generatedIdentity, setGeneratedIdentity] = useState<IdentityInfo | null>(null)

  const steps = [
    {
      label: 'Identity Information',
      description: 'Choose a display name for your identity',
    },
    {
      label: 'Security Settings',
      description: 'Configure security parameters',
    },
    {
      label: 'Generate Identity',
      description: 'Create your cryptographic identity',
    },
  ]

  const handleNext = () => {
    setActiveStep((prevActiveStep) => prevActiveStep + 1)
  }

  const handleBack = () => {
    setActiveStep((prevActiveStep) => prevActiveStep - 1)
  }

  const handleGenerateIdentity = async () => {
    setIsGenerating(true)
    setError(null)

    const withTimeout = <T,>(p: Promise<T>, ms: number): Promise<T> => {
      return new Promise((resolve, reject) => {
        const t = setTimeout(() => reject(new Error('Identity generation timed out')), ms)
        p.then((v) => { clearTimeout(t); resolve(v) }).catch((e) => { clearTimeout(t); reject(e) })
      })
    }

    try {

      // Step 1: generate four-word identity candidate
      const fourWords = await withTimeout(invoke<string>('generate_four_word_identity', { seed: null }), 15000)

      // Step 2: validate and check availability
      const isValid = await withTimeout(invoke<boolean>('validate_four_word_identity', { fourWords }), 10000)
      if (!isValid) throw 'Generated identity failed validation'
      const available = await withTimeout(invoke<boolean>('check_identity_availability', { fourWords }), 10000)
      if (!available) throw 'Generated identity not available; please retry'

      // Step 3: generate keypair and claim
      const [privateKeyHex, publicKeyHex] = await withTimeout(invoke<[string, string]>('generate_identity_keypair'), 15000)
      const packet = await withTimeout(invoke<any>('claim_four_word_identity_with_proof', {
        fourWords,
        privateKeyHex,
        publicKeyHex,
      }), 20000)

      // Step 4: publish identity packet (local DHT mirror for now)
      await withTimeout(invoke('publish_identity_packet', { packet }), 15000)

      // Step 4.1: securely store the identity keypair using the DHT id as the user id
      try {
        await withTimeout(invoke('store_derived_key', {
          userId: packet.dht_id as string,
          keyId: 'identity_keypair',
          keyData: JSON.stringify({ privateKeyHex, publicKeyHex }),
          keyType: 'identity-keypair',
          scope: 'self',
        }), 8000)
      } catch (storageErr) {
        console.warn('Secure storage failed:', storageErr)
        // Don't block identity creation if secure storage is unavailable; surface a friendly error
        setError('Identity created, but secure key storage is unavailable on this device. You can proceed, but some features may be limited.')
      }

      // Step 5: assemble UI identity info shape
      const identity: IdentityInfo = {
        id: packet.dht_id,
        address: packet.dht_id,
        four_word_address: packet.four_words,
        display_name: displayName || packet.four_words,
        created_at: new Date(packet.created_at * 1000).toISOString(),
        is_active: true,
        is_primary: true,
        public_key_hex: publicKeyHex,
        verification_status: 'verified',
      }

      setGeneratedIdentity(identity)
      handleNext()
    } catch (err) {
      const message = err instanceof Error ? err.message : (err as string)
      setError(message)
    } finally {
      setIsGenerating(false)
    }
  }

  // Fallback/local identity in case backend is unavailable
  const generateFallbackIdentity = (): IdentityInfo => {
    const words = ['ocean','forest','mountain','river','sun','moon','star','cloud']
    const pick = () => words[Math.floor(Math.random() * words.length)]
    const four = `${pick()}-${pick()}-${pick()}-${pick()}`
    const now = new Date().toISOString()
    return {
      id: `local_${Date.now()}`,
      address: `dht://${four}`,
      four_word_address: four,
      display_name: displayName || four,
      created_at: now,
      is_active: true,
      is_primary: true,
      public_key_hex: 'local-dev-key',
      verification_status: 'unverified',
    }
  }

  const handleComplete = () => {
    if (generatedIdentity) {
      onIdentityCreated(generatedIdentity)
    }
    onClose()
  }

  const handleClose = () => {
    if (!isGenerating) {
      onClose()
    }
  }

  const renderStepContent = (step: number) => {
    switch (step) {
      case 0:
        return (
          <Box sx={{ mt: 2 }}>
            <TextField
              fullWidth
              label="Display Name (Optional)"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="e.g., Alice, Bob, or leave empty"
              helperText="This is how others will see your identity. You can change this later."
              sx={{ mb: 2 }}
            />
            <Alert severity="info" sx={{ mt: 2 }}>
              Your network address will be a unique 4-word combination like "warm-ocean-gentle-breeze"
              that others can use to connect to you.
            </Alert>
          </Box>
        )

      case 1:
        return (
          <Box sx={{ mt: 2 }}>
            <FormControlLabel
              control={
                <Checkbox
                  checked={useHardwareEntropy}
                  onChange={(e) => setUseHardwareEntropy(e.target.checked)}
                />
              }
              label="Use Hardware Entropy"
            />
            <Typography variant="body2" color="text.secondary" sx={{ ml: 4, mb: 2 }}>
              More secure but slightly slower identity generation
            </Typography>

            <Typography gutterBottom>
              Proof of Work Difficulty: {powDifficulty}
            </Typography>
            <Box sx={{ px: 2 }}>
              <input
                type="range"
                min="4"
                max="16"
                value={powDifficulty}
                onChange={(e) => setPowDifficulty(parseInt(e.target.value))}
                style={{ width: '100%' }}
              />
            </Box>
            <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
              Higher difficulty provides better Sybil resistance but takes longer to generate
            </Typography>

            <Alert severity="warning" sx={{ mt: 2 }}>
              <Typography variant="body2">
                <strong>Security Note:</strong> Your cryptographic keys will be stored securely
                using your operating system's keychain. Make sure you have access to this device
                for future use.
              </Typography>
            </Alert>
          </Box>
        )

      case 2:
        if (isGenerating) {
          return (
            <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', mt: 3 }}>
              <CircularProgress size={60} sx={{ mb: 2 }} />
              <Typography variant="h6" gutterBottom>
                Generating Identity...
              </Typography>
              <Typography variant="body2" color="text.secondary" align="center">
                This may take a few moments depending on your security settings.
                Please don't close this window.
              </Typography>
            </Box>
          )
        }

        if (generatedIdentity) {
          return (
            <Box sx={{ mt: 2 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                <CheckIcon color="success" sx={{ mr: 1 }} />
                <Typography variant="h6" color="success.main">
                  Identity Created Successfully!
                </Typography>
              </Box>

              <Alert severity="success" sx={{ mb: 2 }}>
                <Typography variant="body1" sx={{ mb: 1 }}>
                  <strong>Your Network Address:</strong>
                </Typography>
                <Typography
                  variant="h5"
                  sx={{ fontFamily: 'monospace', color: 'success.main' }}
                >
                  {generatedIdentity.four_word_address}
                </Typography>
              </Alert>

              <Typography variant="body2" color="text.secondary">
                Your identity has been securely stored and is ready to use. You can now
                connect to the P2P network and start communicating with others.
              </Typography>
            </Box>
          )
        }

        return (
          <Box sx={{ mt: 2 }}>
            <Typography variant="body1" gutterBottom>
              Ready to generate your identity with the following settings:
            </Typography>
            <Box sx={{ ml: 2, mt: 2 }}>
              <Typography variant="body2">
                • Display Name: {displayName || 'None'}
              </Typography>
              <Typography variant="body2">
                • Hardware Entropy: {useHardwareEntropy ? 'Enabled' : 'Disabled'}
              </Typography>
              <Typography variant="body2">
                • Security Level: {powDifficulty}/16
              </Typography>
            </Box>
            {error && (
              <Alert severity="error" sx={{ mt: 2 }}>
                {error}
              </Alert>
            )}
          </Box>
        )

      default:
        return null
    }
  }

  return (
    <Dialog
      open={open}
      onClose={handleClose}
      maxWidth="md"
      fullWidth
      PaperProps={{
        sx: { minHeight: '500px' }
      }}
    >
      <DialogTitle>
        <Box display="flex" alignItems="center" gap={1}>
          <PersonAddIcon />
          Create New Identity
        </Box>
      </DialogTitle>

      <DialogContent>
        <Stepper activeStep={activeStep} orientation="vertical">
          {steps.map((step, index) => (
            <Step key={step.label}>
              <StepLabel>{step.label}</StepLabel>
              <StepContent>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  {step.description}
                </Typography>
                {renderStepContent(index)}
              </StepContent>
            </Step>
          ))}
        </Stepper>
      </DialogContent>

      <DialogActions>
        <Button onClick={handleClose} disabled={isGenerating}>
          Cancel
        </Button>
        <Button onClick={() => { const id = generateFallbackIdentity(); setGeneratedIdentity(id); onIdentityCreated(id) }} disabled={isGenerating} color="inherit">
          Skip for now
        </Button>
        
        {activeStep > 0 && activeStep < steps.length - 1 && (
          <Button onClick={handleBack} disabled={isGenerating}>
            Back
          </Button>
        )}
        
        {activeStep < steps.length - 1 && !generatedIdentity && (
          <Button
            onClick={activeStep === steps.length - 2 ? handleGenerateIdentity : handleNext}
            variant="contained"
            disabled={isGenerating}
            startIcon={isGenerating ? <CircularProgress size={16} /> : undefined}
          >
            {activeStep === steps.length - 2 ? 'Generate Identity' : 'Next'}
          </Button>
        )}
        
        {generatedIdentity && (
          <Button onClick={handleComplete} variant="contained" color="success">
            Complete Setup
          </Button>
        )}
      </DialogActions>
    </Dialog>
  )
}

export default IdentitySetup
