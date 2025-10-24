/**
 * MessageList Component Tests
 *
 * Tests message rendering, encrypted message handling, causal ordering,
 * and deduplication.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import { MessageList } from '../MessageList';
import type { Message } from '../../types';

describe('MessageList', () => {
  const createMessage = (overrides?: Partial<Message>): Message => ({
    id: `msg-${Date.now()}-${Math.random()}`,
    channelId: 'test-channel',
    content: 'Test message',
    author: 'test-user',
    authorPeerId: 'test-peer-id',
    timestamp: Date.now(),
    vectorClock: { 'test-peer-id': 1 },
    lamportClock: 1,
    encrypted: false,
    ...overrides,
  });

  it('renders list of messages', () => {
    const messages = [
      createMessage({ id: '1', content: 'First message' }),
      createMessage({ id: '2', content: 'Second message' }),
      createMessage({ id: '3', content: 'Third message' }),
    ];

    render(<MessageList messages={messages} />);

    expect(screen.getByText('First message')).toBeInTheDocument();
    expect(screen.getByText('Second message')).toBeInTheDocument();
    expect(screen.getByText('Third message')).toBeInTheDocument();
  });

  it('shows placeholder for encrypted messages', () => {
    const messages = [
      createMessage({ encrypted: true, content: 'encrypted payload' }),
    ];

    render(<MessageList messages={messages} />);

    expect(screen.getByText(/encrypted/i)).toBeInTheDocument();
    expect(screen.queryByText('encrypted payload')).not.toBeInTheDocument();
  });

  it('displays decrypted content when encrypted=false', () => {
    const messages = [
      createMessage({ encrypted: false, content: 'Decrypted message' }),
    ];

    render(<MessageList messages={messages} />);

    expect(screen.getByText('Decrypted message')).toBeInTheDocument();
    expect(screen.queryByText(/encrypted/i)).not.toBeInTheDocument();
  });

  it('shows lock icon for encrypted messages', () => {
    const messages = [
      createMessage({ encrypted: true }),
    ];

    render(<MessageList messages={messages} />);

    const lockIcon = screen.getByTestId('encrypted-icon');
    expect(lockIcon).toBeInTheDocument();
  });

  it('displays author name for each message', () => {
    const messages = [
      createMessage({ author: 'Alice', content: 'Hello' }),
      createMessage({ author: 'Bob', content: 'Hi there' }),
    ];

    render(<MessageList messages={messages} />);

    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.getByText('Bob')).toBeInTheDocument();
  });

  it('displays timestamp for each message', () => {
    const now = Date.now();
    const messages = [
      createMessage({ timestamp: now }),
    ];

    render(<MessageList messages={messages} />);

    // Should display formatted time (e.g., "12:34 PM")
    const timeElement = screen.getByTestId('message-timestamp');
    expect(timeElement).toBeInTheDocument();
  });

  it('preserves causal ordering', () => {
    const messages = [
      createMessage({ 
        id: '1', 
        content: 'First', 
        lamportClock: 1,
        vectorClock: { 'peer-a': 1 }
      }),
      createMessage({ 
        id: '2', 
        content: 'Second', 
        lamportClock: 2,
        vectorClock: { 'peer-a': 2 }
      }),
      createMessage({ 
        id: '3', 
        content: 'Third', 
        lamportClock: 3,
        vectorClock: { 'peer-a': 3 }
      }),
    ];

    const { container } = render(<MessageList messages={messages} />);

    const messageElements = container.querySelectorAll('[data-testid="message-item"]');
    expect(messageElements).toHaveLength(3);
    
    expect(within(messageElements[0]).getByText('First')).toBeInTheDocument();
    expect(within(messageElements[1]).getByText('Second')).toBeInTheDocument();
    expect(within(messageElements[2]).getByText('Third')).toBeInTheDocument();
  });

  it('handles duplicate messages gracefully', () => {
    const duplicateMsg = createMessage({ id: 'dup', content: 'Duplicate' });
    const messages = [duplicateMsg, duplicateMsg];

    render(<MessageList messages={messages} />);

    // Should only render once (component deduplicates by ID)
    const messageElements = screen.getAllByTestId('message-item');
    expect(messageElements).toHaveLength(1);
  });

  it('shows empty state when no messages', () => {
    render(<MessageList messages={[]} />);

    expect(screen.getByText(/no messages/i)).toBeInTheDocument();
  });

  it('auto-scrolls to bottom on new messages', () => {
    const { rerender } = render(<MessageList messages={[createMessage()]} />);

    const scrollContainer = screen.getByTestId('message-scroll-container');
    const scrollSpy = vi.spyOn(scrollContainer, 'scrollTo');

    // Add new message
    const newMessages = [
      createMessage({ id: '1' }),
      createMessage({ id: '2' }),
    ];

    rerender(<MessageList messages={newMessages} />);

    expect(scrollSpy).toHaveBeenCalled();
  });

  it('groups messages by author', () => {
    const messages = [
      createMessage({ author: 'Alice', content: 'Msg 1' }),
      createMessage({ author: 'Alice', content: 'Msg 2' }),
      createMessage({ author: 'Bob', content: 'Msg 3' }),
    ];

    render(<MessageList messages={messages} />);

    // Alice's name should appear only once (grouped)
    const aliceHeaders = screen.getAllByText('Alice');
    expect(aliceHeaders).toHaveLength(1);
  });

  it('displays four-word peer ID when author is missing', () => {
    const messages = [
      createMessage({ 
        author: '', 
        authorPeerId: 'ocean-forest-moon-star',
        content: 'Test' 
      }),
    ];

    render(<MessageList messages={messages} />);

    expect(screen.getByText(/ocean-forest-moon-star/i)).toBeInTheDocument();
  });
});
