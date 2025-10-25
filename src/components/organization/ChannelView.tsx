import {
    MoreVert as MoreIcon, PersonAdd as AddMemberIcon, Phone as PhoneIcon, Reply as ReplyIcon, Send as SendIcon, VideoCall as VideoIcon
} from '@mui/icons-material';
import {
    Avatar, Box, Chip, IconButton, List,
    ListItem, Menu,
    MenuItem, Stack, TextField, Tooltip, Typography
} from '@mui/material';
import { listen } from '@tauri-apps/api/event';
import React, { useEffect, useRef, useState } from 'react';
import { channelService } from '../../services/channelService';
import type { Channel, Message } from '../../types/channels';
import { GlassCard } from '../ui/GlassCard';
import { AddMemberDialog } from '../members/AddMemberDialog';

interface ChannelViewProps {
  channelId: string;
  currentUserId: string;
  onStartCall?: (type: 'voice' | 'video') => void;
}

export const ChannelView: React.FC<ChannelViewProps> = ({
  channelId,
  currentUserId,
  onStartCall,
}) => {
  const [channel, setChannel] = useState<Channel | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [newMessage, setNewMessage] = useState('');
  const [selectedThread, setSelectedThread] = useState<string | null>(null);
  const [threadMessages, setThreadMessages] = useState<Message[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [addMemberDialogOpen, setAddMemberDialogOpen] = useState(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Load channel data
  useEffect(() => {
    loadChannel();
    loadMessages();
  }, [channelId]);

  // Subscribe to real-time updates
  useEffect(() => {
    const unsubscribe = listen<Message>(`channel:${channelId}:message`, (event) => {
      const newMsg = event.payload;
      if (newMsg.thread_id) {
        if (newMsg.thread_id === selectedThread) {
          setThreadMessages((prev) => [...prev, newMsg]);
        }
      } else {
        setMessages((prev) => [...prev, newMsg]);
      }
      scrollToBottom();
    });

    return () => {
      unsubscribe.then((fn) => fn());
    };
  }, [channelId, selectedThread]);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    scrollToBottom();
  }, [messages, threadMessages]);

  const loadChannel = async () => {
    try {
      const ch = await channelService.getChannel(channelId);
      setChannel(ch);
    } catch (error) {
      console.error('Failed to load channel:', error);
    }
  };

  const loadMessages = async () => {
    try {
      const msgs = await channelService.getMessages(channelId, 50);
      setMessages(msgs.reverse()); // Latest at bottom
    } catch (error) {
      console.error('Failed to load messages:', error);
    }
  };

  const loadThreadMessages = async (threadId: string) => {
    try {
      const msgs = await channelService.getThreadReplies(threadId);
      setThreadMessages(msgs);
    } catch (error) {
      console.error('Failed to load thread:', error);
    }
  };

  const handleSend = async () => {
    if (!newMessage.trim()) return;

    setIsLoading(true);
    try {
      const message = await channelService.sendMessage({
        channel_id: channelId,
        author_id: currentUserId,
        content: newMessage.trim(),
        thread_id: selectedThread ?? undefined,
      });

      if (selectedThread) {
        setThreadMessages((prev) => [...prev, message]);
      } else {
        setMessages((prev) => [...prev, message]);
      }

      setNewMessage('');
      inputRef.current?.focus();
    } catch (error) {
      console.error('Failed to send message:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleStartThread = async (messageId: string) => {
    try {
      const thread = await channelService.createThread(messageId);
      setSelectedThread(thread.id);
      await loadThreadMessages(thread.id);
    } catch (error) {
      console.error('Failed to create thread:', error);
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
    });
  };

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (date.toDateString() === today.toDateString()) {
      return 'Today';
    } else if (date.toDateString() === yesterday.toDateString()) {
      return 'Yesterday';
    } else {
      return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    }
  };

  const MessageItem: React.FC<{ message: Message }> = ({ message }) => {
    const isOwn = message.author_id === currentUserId;

    return (
      <ListItem
        sx={{
          flexDirection: isOwn ? 'row-reverse' : 'row',
          gap: 1,
          px: 2,
          alignItems: 'flex-start',
        }}
      >
        {!isOwn && (
          <Avatar sx={{ width: 32, height: 32, mt: 0.5 }}>
            {message.author_id[0].toUpperCase()}
          </Avatar>
        )}

        <Box sx={{ maxWidth: '70%' }}>
          <GlassCard
            variant={isOwn ? 'gradient' : 'light'}
            blur={15}
            hover={false}
            sx={{
              px: 2,
              py: 1.5,
              display: 'inline-block',
            }}
          >
            {!isOwn && (
              <Typography
                variant="caption"
                fontWeight={600}
                display="block"
                sx={{ mb: 0.5, opacity: 0.9 }}
              >
                {message.author_id}
              </Typography>
            )}

            <Typography variant="body2" sx={{ wordBreak: 'break-word' }}>
              {message.content}
            </Typography>

            <Stack
              direction="row"
              alignItems="center"
              spacing={1}
              sx={{ mt: 0.5 }}
            >
              <Typography
                variant="caption"
                sx={{ opacity: 0.7, fontSize: '0.7rem' }}
              >
                {formatTime(message.created_at)}
              </Typography>

              {!isOwn && !message.thread_id && (
                <Tooltip title="Start thread">
                  <IconButton
                    size="small"
                    onClick={() => handleStartThread(message.id)}
                    sx={{ p: 0.25 }}
                  >
                    <ReplyIcon sx={{ fontSize: 14 }} />
                  </IconButton>
                </Tooltip>
              )}
            </Stack>
          </GlassCard>
        </Box>
      </ListItem>
    );
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Channel Header */}
      <GlassCard
        variant="light"
        blur={20}
        hover={false}
        sx={{
          px: 3,
          py: 2,
          borderRadius: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <Stack direction="row" spacing={2} alignItems="center">
          <Box
            sx={{
              width: 40,
              height: 40,
              borderRadius: 2,
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: '#fff',
              fontSize: '1.25rem',
              fontWeight: 600,
            }}
          >
            #
          </Box>
          <Box>
            <Typography variant="h6" fontWeight={600}>
              {channel?.name || 'Loading...'}
            </Typography>
            {channel?.description && (
              <Typography variant="caption" color="text.secondary">
                {channel.description}
              </Typography>
            )}
          </Box>
        </Stack>

        <Stack direction="row" spacing={1}>
          <Tooltip title="Voice call">
            <IconButton size="small" onClick={() => onStartCall?.('voice')}>
              <PhoneIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="Video call">
            <IconButton size="small" onClick={() => onStartCall?.('video')}>
              <VideoIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <Tooltip title="Add members">
            <IconButton 
              size="small"
              onClick={() => setAddMemberDialogOpen(true)}
            >
              <AddMemberIcon fontSize="small" />
            </IconButton>
          </Tooltip>
          <IconButton
            size="small"
            onClick={(e) => setMenuAnchor(e.currentTarget)}
          >
            <MoreIcon fontSize="small" />
          </IconButton>
        </Stack>

        <Menu
          anchorEl={menuAnchor}
          open={Boolean(menuAnchor)}
          onClose={() => setMenuAnchor(null)}
        >
          <MenuItem onClick={() => setMenuAnchor(null)}>
            Channel settings
          </MenuItem>
          <MenuItem onClick={() => setMenuAnchor(null)}>
            Notification preferences
          </MenuItem>
          <MenuItem onClick={() => setMenuAnchor(null)}>
            Pin channel
          </MenuItem>
        </Menu>
      </GlassCard>

      {/* Messages Area */}
      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
          px: 1,
          py: 2,
        }}
      >
        <List sx={{ py: 0 }}>
          {messages.map((msg, idx) => {
            const prevMsg = idx > 0 ? messages[idx - 1] : null;
            const showDate =
              !prevMsg ||
              formatDate(msg.created_at) !== formatDate(prevMsg.created_at);

            return (
              <React.Fragment key={msg.id}>
                {showDate && (
                  <Box sx={{ textAlign: 'center', my: 2 }}>
                    <Chip
                      label={formatDate(msg.created_at)}
                      size="small"
                      sx={{
                        background: 'rgba(255,255,255,0.1)',
                        backdropFilter: 'blur(10px)',
                      }}
                    />
                  </Box>
                )}
                <MessageItem message={msg} />
              </React.Fragment>
            );
          })}
        </List>
        <div ref={messagesEndRef} />
      </Box>

      {/* Thread Sidebar */}
      {selectedThread && (
        <GlassCard
          variant="dark"
          blur={30}
          hover={false}
          sx={{
            position: 'absolute',
            right: 0,
            top: 80,
            bottom: 80,
            width: 400,
            display: 'flex',
            flexDirection: 'column',
            zIndex: 10,
          }}
        >
          <Box sx={{ p: 2, borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
            <Stack
              direction="row"
              alignItems="center"
              justifyContent="space-between"
            >
              <Typography variant="h6">Thread</Typography>
              <IconButton size="small" onClick={() => setSelectedThread(null)}>
                ✕
              </IconButton>
            </Stack>
          </Box>

          <Box sx={{ flex: 1, overflow: 'auto', px: 2, py: 1 }}>
            <List>
              {threadMessages.map((msg) => (
                <MessageItem key={msg.id} message={msg} />
              ))}
            </List>
          </Box>

          <Box sx={{ p: 2, borderTop: '1px solid rgba(255,255,255,0.1)' }}>
            <TextField
              fullWidth
              size="small"
              placeholder="Reply..."
              value={newMessage}
              onChange={(e) => setNewMessage(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && !e.shiftKey && handleSend()}
            />
          </Box>
        </GlassCard>
      )}

      {/* Message Input */}
      <GlassCard
        variant="light"
        blur={20}
        hover={false}
        sx={{
          px: 3,
          py: 2,
          borderRadius: 0,
        }}
      >
        <Stack direction="row" spacing={1.5} alignItems="center">
          <TextField
            fullWidth
            multiline
            maxRows={4}
            placeholder={`Message #${channel?.name || '...'}`}
            value={newMessage}
            onChange={(e) => setNewMessage(e.target.value)}
            onKeyPress={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            inputRef={inputRef}
            disabled={isLoading}
            sx={{
              '& .MuiOutlinedInput-root': {
                background: 'rgba(255, 255, 255, 0.05)',
                backdropFilter: 'blur(10px)',
                borderRadius: 3,
                '& fieldset': {
                  borderColor: 'rgba(255, 255, 255, 0.1)',
                },
              },
            }}
          />

          <Tooltip title="Send message">
            <IconButton
              color="primary"
              onClick={handleSend}
              disabled={!newMessage.trim() || isLoading}
              sx={{
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                color: '#fff',
                '&:hover': {
                  background: 'linear-gradient(135deg, #764ba2 0%, #667eea 100%)',
                },
              }}
            >
              <SendIcon />
            </IconButton>
          </Tooltip>
        </Stack>
      </GlassCard>

      {/* Member Management Dialog */}
      <AddMemberDialog
        open={addMemberDialogOpen}
        onClose={() => setAddMemberDialogOpen(false)}
        entityType="channel"
        entityId={channelId}
        onMemberAdded={() => {
          // Member added successfully
          console.log('Member added to channel', channelId)
          // Could refresh channel data here if needed
        }}
      />
    </Box>
  );
};
