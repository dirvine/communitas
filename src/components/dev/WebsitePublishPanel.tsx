import React, { useCallback, useMemo, useState } from 'react'
import { Box, Button, TextField, Typography, Alert, Stack, Divider } from '@mui/material'
import { buildWebsiteRootCanonicalHex, applyWebsiteRootWithSignature, publishWebsiteAndSetRoot } from '../../services/website'
import { invoke } from '@tauri-apps/api/tauri'

export const WebsitePublishPanel: React.FC = () => {
  const [entityHex, setEntityHex] = useState('')
  const [websiteRootHex, setWebsiteRootHex] = useState('')
  const [pkHex, setPkHex] = useState('')
  const [sigHex, setSigHex] = useState('')
  const [canonicalHex, setCanonicalHex] = useState('')
  const [busy, setBusy] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [receipt, setReceipt] = useState<any>(null)

  const disabled = useMemo(() => busy, [busy])

  const onGenerateCanonical = useCallback(() => {
    try {
      setError(null); setMessage(null)
      if (!entityHex || !pkHex || !websiteRootHex) {
        setError('Please provide Entity ID (hex), Public Key (hex), and Website Root (hex).')
        return
      }
      const hex = buildWebsiteRootCanonicalHex(entityHex.trim(), pkHex.trim(), websiteRootHex.trim())
      setCanonicalHex(hex)
      setMessage('Canonical bytes (hex) generated. Sign these bytes with your ML‑DSA secret key and paste signature hex below.')
    } catch (e: any) {
      setError(`Failed to build canonical hex: ${e?.message || e}`)
    }
  }, [entityHex, pkHex, websiteRootHex])

  const onPublishOnly = useCallback(async () => {
    setBusy(true); setError(null); setMessage(null); setReceipt(null)
    try {
      const rcpt = await invoke('core_website_publish_receipt', { entityHex: entityHex.trim(), websiteRootHex: websiteRootHex.trim() })
      setReceipt(rcpt)
      setMessage('Website published (manifest stored).')
    } catch (e: any) {
      setError(`Publish failed: ${e?.message || e}`)
    } finally {
      setBusy(false)
    }
  }, [entityHex, websiteRootHex])

  const onApplyUpdateOnly = useCallback(async () => {
    setBusy(true); setError(null); setMessage(null)
    try {
      if (!sigHex) {
        setError('Please paste signature hex before applying update.')
        setBusy(false)
        return
      }
      await applyWebsiteRootWithSignature(entityHex.trim(), websiteRootHex.trim(), sigHex.trim())
      setMessage('identity.website_root updated successfully.')
    } catch (e: any) {
      setError(`Website root update failed: ${e?.message || e}`)
    } finally {
      setBusy(false)
    }
  }, [entityHex, websiteRootHex, sigHex])

  const onPublishAndUpdate = useCallback(async () => {
    setBusy(true); setError(null); setMessage(null); setReceipt(null)
    try {
      const signer = async (_canonicalHex: string) => {
        // We rely on a pre-generated signature pasted by the user.
        if (!sigHex) throw new Error('Signature hex not provided. Generate canonical hex, sign it externally, then paste the signature.')
        return sigHex.trim()
      }
      const result = await publishWebsiteAndSetRoot(entityHex.trim(), websiteRootHex.trim(), signer, pkHex.trim())
      setMessage(`Published=${result.published}, identity updated=${result.updatedIdentity}`)
      const rcpt = await invoke('core_website_publish_receipt', { entityHex: entityHex.trim(), websiteRootHex: websiteRootHex.trim() })
      setReceipt(rcpt)
    } catch (e: any) {
      setError(`Publish+Update failed: ${e?.message || e}`)
    } finally {
      setBusy(false)
    }
  }, [entityHex, websiteRootHex, pkHex, sigHex])

  return (
    <Box sx={{ p: 3, maxWidth: 900, mx: 'auto' }}>
      <Typography variant="h5" gutterBottom>Website Publishing Utility</Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
        Use this panel to publish a website container (manifest) and update identity.website_root.
        To update identity, you must sign canonical bytes with the ML‑DSA secret key.
      </Typography>

      <Stack spacing={2}>
        <TextField
          label="Entity ID (hex, 32 bytes)"
          value={entityHex}
          onChange={e => setEntityHex(e.target.value)}
          fullWidth
          size="small"
        />
        <TextField
          label="Website Root (hex, 32 bytes)"
          value={websiteRootHex}
          onChange={e => setWebsiteRootHex(e.target.value)}
          fullWidth
          size="small"
        />
        <TextField
          label="Public Key (hex, ML‑DSA pub key)"
          value={pkHex}
          onChange={e => setPkHex(e.target.value)}
          fullWidth
          size="small"
        />

        <Stack direction="row" spacing={1}>
          <Button variant="outlined" onClick={onGenerateCanonical} disabled={disabled}>Generate Canonical Bytes</Button>
          <Button variant="contained" onClick={onPublishOnly} disabled={disabled}>Publish Website</Button>
          <Button variant="contained" color="secondary" onClick={onApplyUpdateOnly} disabled={disabled}>Apply Identity Update</Button>
          <Button variant="contained" color="success" onClick={onPublishAndUpdate} disabled={disabled}>Publish + Update</Button>
        </Stack>

        <TextField
          label="Canonical Bytes (hex)"
          value={canonicalHex}
          onChange={e => setCanonicalHex(e.target.value)}
          fullWidth
          size="small"
          multiline
          minRows={3}
        />

        <TextField
          label="Signature Hex (ML‑DSA signature over canonical)"
          value={sigHex}
          onChange={e => setSigHex(e.target.value)}
          fullWidth
          size="small"
          multiline
          minRows={2}
        />

        {message && <Alert severity="success">{message}</Alert>}
        {error && <Alert severity="error">{error}</Alert>}

        {receipt && (
          <>
            <Divider />
            <Typography variant="subtitle1">Publish Receipt</Typography>
            <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word', background: '#0b0f14', color: '#cde', padding: 12, borderRadius: 6 }}>
              {JSON.stringify(receipt, null, 2)}
            </pre>
          </>
        )}
      </Stack>
    </Box>
  )
}

export default WebsitePublishPanel

