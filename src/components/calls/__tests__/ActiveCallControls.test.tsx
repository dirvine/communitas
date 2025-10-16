import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ActiveCallControls } from '../ActiveCallControls';

describe('ActiveCallControls', () => {
  const mockOnVideoToggle = vi.fn();
  const mockOnAudioToggle = vi.fn();
  const mockOnScreenShareToggle = vi.fn();
  const mockOnEndCall = vi.fn();

  const defaultProps = {
    callId: 'test-call-id',
    isVideoEnabled: false,
    isAudioEnabled: false,
    isScreenSharing: false,
    onVideoToggle: mockOnVideoToggle,
    onAudioToggle: mockOnAudioToggle,
    onScreenShareToggle: mockOnScreenShareToggle,
    onEndCall: mockOnEndCall,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render all control buttons', () => {
      render(<ActiveCallControls {...defaultProps} />);

      expect(screen.getByRole('button', { name: /video/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /audio|mic/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /screen/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /end|hang/i })).toBeInTheDocument();
    });

    it('should show disabled state for video button when video is off', () => {
      render(<ActiveCallControls {...defaultProps} isVideoEnabled={false} />);

      const videoButton = screen.getByRole('button', { name: /video/i });
      expect(videoButton).toHaveAttribute('aria-pressed', 'false');
    });

    it('should show enabled state for video button when video is on', () => {
      render(<ActiveCallControls {...defaultProps} isVideoEnabled={true} />);

      const videoButton = screen.getByRole('button', { name: /video/i });
      expect(videoButton).toHaveAttribute('aria-pressed', 'true');
    });

    it('should show muted state for audio button when audio is off', () => {
      render(<ActiveCallControls {...defaultProps} isAudioEnabled={false} />);

      const audioButton = screen.getByRole('button', { name: /audio|mic/i });
      expect(audioButton).toHaveAttribute('aria-pressed', 'false');
    });

    it('should show unmuted state for audio button when audio is on', () => {
      render(<ActiveCallControls {...defaultProps} isAudioEnabled={true} />);

      const audioButton = screen.getByRole('button', { name: /audio|mic/i });
      expect(audioButton).toHaveAttribute('aria-pressed', 'true');
    });

    it('should show inactive state for screen share button when not sharing', () => {
      render(<ActiveCallControls {...defaultProps} isScreenSharing={false} />);

      const screenButton = screen.getByRole('button', { name: /screen/i });
      expect(screenButton).toHaveAttribute('aria-pressed', 'false');
    });

    it('should show active state for screen share button when sharing', () => {
      render(<ActiveCallControls {...defaultProps} isScreenSharing={true} />);

      const screenButton = screen.getByRole('button', { name: /screen/i });
      expect(screenButton).toHaveAttribute('aria-pressed', 'true');
    });
  });

  describe('Video Control', () => {
    it('should call onVideoToggle when video button is clicked', () => {
      render(<ActiveCallControls {...defaultProps} />);

      const videoButton = screen.getByRole('button', { name: /video/i });
      fireEvent.click(videoButton);

      expect(mockOnVideoToggle).toHaveBeenCalledWith(true);
    });

    it('should toggle from enabled to disabled', () => {
      render(<ActiveCallControls {...defaultProps} isVideoEnabled={true} />);

      const videoButton = screen.getByRole('button', { name: /video/i });
      fireEvent.click(videoButton);

      expect(mockOnVideoToggle).toHaveBeenCalledWith(false);
    });

    it('should disable button while toggling', async () => {
      mockOnVideoToggle.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );

      render(<ActiveCallControls {...defaultProps} />);

      const videoButton = screen.getByRole('button', { name: /video/i });
      fireEvent.click(videoButton);

      // Button should be disabled during operation
      expect(videoButton).toBeDisabled();

      await waitFor(() => {
        expect(videoButton).not.toBeDisabled();
      });
    });
  });

  describe('Audio Control', () => {
    it('should call onAudioToggle when audio button is clicked', () => {
      render(<ActiveCallControls {...defaultProps} />);

      const audioButton = screen.getByRole('button', { name: /audio|mic/i });
      fireEvent.click(audioButton);

      expect(mockOnAudioToggle).toHaveBeenCalledWith(true);
    });

    it('should toggle from enabled to disabled', () => {
      render(<ActiveCallControls {...defaultProps} isAudioEnabled={true} />);

      const audioButton = screen.getByRole('button', { name: /audio|mic/i });
      fireEvent.click(audioButton);

      expect(mockOnAudioToggle).toHaveBeenCalledWith(false);
    });

    it('should disable button while toggling', async () => {
      mockOnAudioToggle.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );

      render(<ActiveCallControls {...defaultProps} />);

      const audioButton = screen.getByRole('button', { name: /audio|mic/i });
      fireEvent.click(audioButton);

      expect(audioButton).toBeDisabled();

      await waitFor(() => {
        expect(audioButton).not.toBeDisabled();
      });
    });
  });

  describe('Screen Share Control', () => {
    it('should call onScreenShareToggle when screen share button is clicked', () => {
      render(<ActiveCallControls {...defaultProps} />);

      const screenButton = screen.getByRole('button', { name: /screen/i });
      fireEvent.click(screenButton);

      expect(mockOnScreenShareToggle).toHaveBeenCalledWith(true);
    });

    it('should toggle from active to inactive', () => {
      render(<ActiveCallControls {...defaultProps} isScreenSharing={true} />);

      const screenButton = screen.getByRole('button', { name: /screen/i });
      fireEvent.click(screenButton);

      expect(mockOnScreenShareToggle).toHaveBeenCalledWith(false);
    });

    it('should disable button while toggling', async () => {
      mockOnScreenShareToggle.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );

      render(<ActiveCallControls {...defaultProps} />);

      const screenButton = screen.getByRole('button', { name: /screen/i });
      fireEvent.click(screenButton);

      expect(screenButton).toBeDisabled();

      await waitFor(() => {
        expect(screenButton).not.toBeDisabled();
      });
    });
  });

  describe('End Call Control', () => {
    it('should call onEndCall when end call button is clicked', () => {
      render(<ActiveCallControls {...defaultProps} />);

      const endButton = screen.getByRole('button', { name: /end|hang/i });
      fireEvent.click(endButton);

      expect(mockOnEndCall).toHaveBeenCalledTimes(1);
    });

    it('should disable button while ending call', async () => {
      mockOnEndCall.mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );

      render(<ActiveCallControls {...defaultProps} />);

      const endButton = screen.getByRole('button', { name: /end|hang/i });
      fireEvent.click(endButton);

      expect(endButton).toBeDisabled();

      await waitFor(() => {
        expect(endButton).not.toBeDisabled();
      });
    });
  });

  describe('Accessibility', () => {
    it('should have proper ARIA labels', () => {
      render(<ActiveCallControls {...defaultProps} />);

      expect(screen.getByRole('button', { name: /video/i })).toHaveAttribute('aria-label');
      expect(screen.getByRole('button', { name: /audio|mic/i })).toHaveAttribute('aria-label');
      expect(screen.getByRole('button', { name: /screen/i })).toHaveAttribute('aria-label');
      expect(screen.getByRole('button', { name: /end|hang/i })).toHaveAttribute('aria-label');
    });

    it('should have proper pressed states', () => {
      render(
        <ActiveCallControls
          {...defaultProps}
          isVideoEnabled={true}
          isAudioEnabled={true}
          isScreenSharing={true}
        />
      );

      expect(screen.getByRole('button', { name: /video/i })).toHaveAttribute(
        'aria-pressed',
        'true'
      );
      expect(screen.getByRole('button', { name: /audio|mic/i })).toHaveAttribute(
        'aria-pressed',
        'true'
      );
      expect(screen.getByRole('button', { name: /screen/i })).toHaveAttribute(
        'aria-pressed',
        'true'
      );
    });

    it('should be keyboard navigable', () => {
      render(<ActiveCallControls {...defaultProps} />);

      const buttons = screen.getAllByRole('button');
      buttons.forEach((button) => {
        expect(button).toHaveAttribute('tabIndex', '0');
      });
    });
  });

  describe('Error Handling', () => {
    it('should handle video toggle errors gracefully', async () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
      mockOnVideoToggle.mockRejectedValueOnce(new Error('Video toggle failed'));

      render(<ActiveCallControls {...defaultProps} />);

      const videoButton = screen.getByRole('button', { name: /video/i });
      fireEvent.click(videoButton);

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalled();
      });

      consoleError.mockRestore();
    });

    it('should handle audio toggle errors gracefully', async () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
      mockOnAudioToggle.mockRejectedValueOnce(new Error('Audio toggle failed'));

      render(<ActiveCallControls {...defaultProps} />);

      const audioButton = screen.getByRole('button', { name: /audio|mic/i });
      fireEvent.click(audioButton);

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalled();
      });

      consoleError.mockRestore();
    });

    it('should handle screen share errors gracefully', async () => {
      const consoleError = vi.spyOn(console, 'error').mockImplementation(() => {});
      mockOnScreenShareToggle.mockRejectedValueOnce(new Error('Screen share failed'));

      render(<ActiveCallControls {...defaultProps} />);

      const screenButton = screen.getByRole('button', { name: /screen/i });
      fireEvent.click(screenButton);

      await waitFor(() => {
        expect(consoleError).toHaveBeenCalled();
      });

      consoleError.mockRestore();
    });
  });
});
