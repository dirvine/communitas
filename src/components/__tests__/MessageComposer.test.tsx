/**
 * MessageComposer Component Tests
 *
 * Tests message composition UI, send button state, keyboard shortcuts,
 * and error handling for IPC failures.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MessageComposer } from '../MessageComposer';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('MessageComposer', () => {
  const mockOnSend = vi.fn();
  const mockChannelId = 'test-channel-123';

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders message input textarea', () => {
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    expect(textarea).toBeInTheDocument();
    expect(textarea).toHaveAttribute('placeholder');
  });

  it('send button is disabled when message is empty', () => {
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    expect(sendButton).toBeDisabled();
  });

  it('send button is enabled when message has content', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Hello world');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    expect(sendButton).toBeEnabled();
  });

  it('calls onSend when send button is clicked', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Test message');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    await user.click(sendButton);
    
    expect(mockOnSend).toHaveBeenCalledWith('Test message');
  });

  it('clears textarea after sending message', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox') as HTMLTextAreaElement;
    await user.type(textarea, 'Test message');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    await user.click(sendButton);
    
    await waitFor(() => {
      expect(textarea.value).toBe('');
    });
  });

  it('sends message on Enter key press', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Quick message{Enter}');
    
    expect(mockOnSend).toHaveBeenCalledWith('Quick message');
  });

  it('does not send on Shift+Enter (multiline)', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox') as HTMLTextAreaElement;
    await user.type(textarea, 'Line 1{Shift>}{Enter}{/Shift}Line 2');
    
    expect(mockOnSend).not.toHaveBeenCalled();
    expect(textarea.value).toContain('\n');
  });

  it('shows error when IPC send fails', async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    vi.mocked(invoke).mockRejectedValueOnce(new Error('IPC failed'));
    
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Test message');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    await user.click(sendButton);
    
    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent(/failed to send/i);
    });
  });

  it('disables send button while sending', async () => {
    const user = userEvent.setup();
    const slowOnSend = vi.fn(() => new Promise(resolve => setTimeout(resolve, 100)));
    
    render(<MessageComposer channelId={mockChannelId} onSend={slowOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Test');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    await user.click(sendButton);
    
    // Button should be disabled during send
    expect(sendButton).toBeDisabled();
    
    await waitFor(() => {
      expect(sendButton).toBeEnabled();
    });
  });

  it('trims whitespace from messages', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, '  Test message  ');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    await user.click(sendButton);
    
    expect(mockOnSend).toHaveBeenCalledWith('Test message');
  });

  it('does not send empty or whitespace-only messages', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, '   ');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    // Should still be disabled
    expect(sendButton).toBeDisabled();
  });

  it('supports multiline messages', async () => {
    const user = userEvent.setup();
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} />);
    
    const textarea = screen.getByRole('textbox');
    await user.type(textarea, 'Line 1{Shift>}{Enter}{/Shift}Line 2{Shift>}{Enter}{/Shift}Line 3');
    
    const sendButton = screen.getByRole('button', { name: /send/i });
    await user.click(sendButton);
    
    expect(mockOnSend).toHaveBeenCalledWith('Line 1\nLine 2\nLine 3');
  });

  it('focuses textarea on mount', () => {
    render(<MessageComposer channelId={mockChannelId} onSend={mockOnSend} autoFocus />);
    
    const textarea = screen.getByRole('textbox');
    expect(textarea).toHaveFocus();
  });
});
