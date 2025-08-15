import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  Box,
  Paper,
  TextField,
  IconButton,
  Typography,
  Stack,
  Avatar,
  Chip,
  Alert,
  CircularProgress,
  Tooltip,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
} from '@mui/material';
import {
  Send as SendIcon,
  Lock as LockIcon,
  LockOpen as LockOpenIcon,
  Key as KeyIcon,
  Security as SecurityIcon,
  Warning as WarningIcon,
  VpnKey as VpnKeyIcon,
  Shield as ShieldIcon,
  MoreVert as MoreIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { useEncryption } from '../../contexts/EncryptionContext';
import { useAuth } from '../../contexts/AuthContext';
import { invoke } from '@tauri-apps/api/core';

export interface SecureMessage {
  id: string;
  senderId: string;
  senderName: string;
  senderAvatar?: string;
  recipientId: string;
  content: string;
  encrypted: boolean;
  keyId?: string;
  timestamp: string;
  signature?: string;
  verified?: boolean;
}

export interface SecureMessagingProps {
  recipientId: string;
  recipientName: string;
  recipientPublicKey?: string;
  organizationId?: string;
  projectId?: string;
  onMessageSent?: (message: SecureMessage) => void;
  onError?: (error: string) => void;
  encryptByDefault?: boolean;
  showEncryptionStatus?: boolean;
}

export const SecureMessaging: React.FC<SecureMessagingProps> = ({
  recipientId,
  recipientName,
  recipientPublicKey,
  organizationId,
  projectId,
  onMessageSent,
  onError,
  encryptByDefault = true,
  showEncryptionStatus = true,
}) => {
  const { 
    state: encryptionState, 
    encryptText, 
    decryptText, 
    deriveSharedKey,
    getOrCreateKey,
    exportPublicKey,
  } = useEncryption();
  const { authState } = useAuth();

  const [messages, setMessages] = useState<SecureMessage[]>([]);
  const [messageText, setMessageText] = useState('');
  const [sending, setSending] = useState(false);
  const [encryptMessages, setEncryptMessages] = useState(encryptByDefault);
  const [sharedKeyEstablished, setSharedKeyEstablished] = useState(false);
  const [loading, setLoading] = useState(false);
  const [menuAnchorEl, setMenuAnchorEl] = useState<HTMLElement | null>(null);

  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Scroll to bottom when new messages arrive
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Establish shared key if recipient public key is available
  useEffect(() => {
    if (recipientPublicKey && encryptionState.userKeyPair && !sharedKeyEstablished) {
      establishSharedKey();
    }
  }, [recipientPublicKey, encryptionState.userKeyPair]);

  const establishSharedKey = useCallback(async () => {
    if (!recipientPublicKey) {
      onError?.('Recipient public key not available for end-to-end encryption');
      return;
    }

    setLoading(true);
    try {
      await deriveSharedKey(recipientPublicKey, `message-${recipientId}`);
      setSharedKeyEstablished(true);
    } catch (error) {
      console.error('Failed to establish shared key:', error);
      onError?.('Failed to establish secure connection');
    } finally {
      setLoading(false);
    }
  }, [recipientPublicKey, recipientId, deriveSharedKey]);

  const sendMessage = useCallback(async () => {
    if (!messageText.trim() || !authState.user) return;

    setSending(true);
    try {
      let encryptedContent = messageText;
      let keyId: string | undefined;
      let encrypted = false;

      if (encryptMessages && encryptionState.isInitialized) {
        if (sharedKeyEstablished) {
          // Use shared key for end-to-end encryption
          const sharedKey = Array.from(encryptionState.sharedKeys.values())
            .find(key => key.purpose.includes(`message-${recipientId}`));
          
          if (sharedKey) {
            encryptedContent = await encryptText(messageText, sharedKey.keyId);
            keyId = sharedKey.keyId;
            encrypted = true;
          }
        } else {
          // Use organization/project key
          const scope = projectId ? `project:${projectId}` : 
                       organizationId ? `organization:${organizationId}` : undefined;
          const messageKey = await getOrCreateKey('message', scope);
          
          encryptedContent = await encryptText(messageText, messageKey.id);
          keyId = messageKey.id;
          encrypted = true;
        }
      }

      const messageId = crypto.randomUUID();
      const message: SecureMessage = {
        id: messageId,
        senderId: authState.user.id,
        senderName: authState.user.name,
        senderAvatar: authState.user.avatar,
        recipientId,
        content: encryptedContent,
        encrypted,
        keyId,
        timestamp: new Date().toISOString(),
      };

      // Store message via Tauri backend
      await invoke('send_message', {
        messageId,
        recipientId,
        content: encryptedContent,
        encrypted,
        keyId,
        organizationId,
        projectId,
      });

      setMessages(prev => [...prev, message]);
      onMessageSent?.(message);
      setMessageText('');

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to send message';
      onError?.(errorMessage);
      console.error('Message send failed:', error);
    } finally {
      setSending(false);
    }
  }, [
    messageText, 
    authState.user, 
    encryptMessages, 
    encryptionState.isInitialized,
    sharedKeyEstablished,
    recipientId,
    organizationId,
    projectId
  ]);

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  const getEncryptionStatusColor = (): "success" | "warning" | "error" => {
    if (!encryptMessages) return 'warning';
    if (sharedKeyEstablished) return 'success';
    if (encryptionState.isInitialized) return 'success';
    return 'error';
  };

  const getEncryptionStatusText = (): string => {
    if (!encryptMessages) return 'Unencrypted';
    if (sharedKeyEstablished) return 'End-to-End Encrypted';
    if (encryptionState.isInitialized) return 'Encrypted';
    return 'Encryption Error';
  };

  const getEncryptionIcon = () => {
    if (loading) return <CircularProgress size={16} />;
    if (!encryptMessages) return <LockOpenIcon />;
    if (sharedKeyEstablished) return <ShieldIcon />;
    if (encryptionState.isInitialized) return <LockIcon />;
    return <WarningIcon />;
  };

  const toggleEncryption = () => {
    setEncryptMessages(!encryptMessages);
    setMenuAnchorEl(null);
  };

  const sharePublicKey = async () => {
    try {
      const publicKey = await exportPublicKey();
      // Copy to clipboard or show in dialog
      navigator.clipboard.writeText(publicKey);
      // Could show a success message
    } catch (error) {
      onError?.('Failed to export public key');
    }
    setMenuAnchorEl(null);
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header with encryption status */}
      {showEncryptionStatus && (
        <Paper sx={{ p: 2, mb: 1 }}>
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Stack direction="row" alignItems="center" spacing={2}>
              <Typography variant="h6">
                Chat with {recipientName}
              </Typography>
              
              {encryptionState.isInitialized && (
                <Chip
                  icon={getEncryptionIcon()}
                  label={getEncryptionStatusText()}
                  color={getEncryptionStatusColor()}
                  size="small"
                />
              )}
            </Stack>

            <IconButton
              size="small"
              onClick={(e) => setMenuAnchorEl(e.currentTarget)}
            >
              <MoreIcon />
            </IconButton>
          </Stack>

          {!sharedKeyEstablished && recipientPublicKey && encryptMessages && (
            <Alert severity="info" sx={{ mt: 1 }}>
              <Stack direction="row" alignItems="center" spacing={1}>
                <KeyIcon fontSize="small" />
                <Typography variant="body2">
                  Establishing secure connection...
                </Typography>
              </Stack>
            </Alert>
          )}
        </Paper>
      )}

      {/* Messages Area */}
      <Paper sx={{ flex: 1, p: 2, overflow: 'auto', mb: 1 }}>
        <Stack spacing={2}>
          <AnimatePresence>
            {messages.map((message) => (
              <motion.div
                key={message.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -20 }}
              >
                <Box
                  sx={{
                    display: 'flex',
                    justifyContent: message.senderId === authState.user?.id ? 'flex-end' : 'flex-start',
                    mb: 1,
                  }}
                >
                  <Paper
                    sx={{
                      maxWidth: '70%',
                      p: 2,
                      bgcolor: message.senderId === authState.user?.id 
                        ? 'primary.main' 
                        : 'background.default',
                      color: message.senderId === authState.user?.id 
                        ? 'primary.contrastText' 
                        : 'text.primary',
                    }}
                  >
                    <Stack spacing={1}>
                      <Stack direction="row" alignItems="center" spacing={1}>
                        <Avatar
                          src={message.senderAvatar}
                          sx={{ width: 24, height: 24 }}
                        >
                          {message.senderName.charAt(0)}
                        </Avatar>
                        <Typography variant="caption" opacity={0.8}>
                          {message.senderName}
                        </Typography>
                        {message.encrypted && (
                          <LockIcon sx={{ fontSize: 14, opacity: 0.8 }} />
                        )}
                      </Stack>
                      
                      <Typography variant="body1">
                        {message.content}
                      </Typography>
                      
                      <Typography variant="caption" opacity={0.6}>
                        {new Date(message.timestamp).toLocaleTimeString()}
                      </Typography>
                    </Stack>
                  </Paper>
                </Box>
              </motion.div>
            ))}
          </AnimatePresence>
          <div ref={messagesEndRef} />
        </Stack>
      </Paper>

      {/* Message Input */}
      <Paper sx={{ p: 2 }}>
        <Stack direction="row" spacing={1} alignItems="flex-end">
          <TextField
            fullWidth
            multiline
            maxRows={4}
            placeholder="Type a message..."
            value={messageText}
            onChange={(e) => setMessageText(e.target.value)}
            onKeyPress={handleKeyPress}
            disabled={sending}
            variant="outlined"
            size="small"
          />
          
          <Tooltip title="Send message">
            <IconButton
              color="primary"
              onClick={sendMessage}
              disabled={!messageText.trim() || sending}
              size="large"
            >
              {sending ? <CircularProgress size={24} /> : <SendIcon />}
            </IconButton>
          </Tooltip>
        </Stack>
      </Paper>

      {/* Settings Menu */}
      <Menu
        anchorEl={menuAnchorEl}
        open={Boolean(menuAnchorEl)}
        onClose={() => setMenuAnchorEl(null)}
      >
        <MenuItem onClick={toggleEncryption}>
          <ListItemIcon>
            {encryptMessages ? <LockOpenIcon /> : <LockIcon />}
          </ListItemIcon>
          <ListItemText>
            {encryptMessages ? 'Disable' : 'Enable'} Encryption
          </ListItemText>
        </MenuItem>
        
        {encryptionState.userKeyPair && (
          <MenuItem onClick={sharePublicKey}>
            <ListItemIcon>
              <VpnKeyIcon />
            </ListItemIcon>
            <ListItemText>
              Share Public Key
            </ListItemText>
          </MenuItem>
        )}
      </Menu>
    </Box>
  );
};

export default SecureMessaging;