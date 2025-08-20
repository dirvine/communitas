import React, { Component, ErrorInfo, ReactNode } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  Alert,
  AlertTitle,
  useTheme,
  alpha,
} from '@mui/material';
import { Error as ErrorIcon, Refresh as RefreshIcon } from '@mui/icons-material';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught an error:', error, errorInfo);

    this.setState({
      error,
      errorInfo,
    });

    // Call optional error handler
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }

    // In production, you might want to send this to an error reporting service
    if (process.env.NODE_ENV === 'production') {
      // Example: sendErrorToService(error, errorInfo);
    }
  }

  handleRetry = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined });
  };

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return <ErrorFallback
        error={this.state.error}
        errorInfo={this.state.errorInfo}
        onRetry={this.handleRetry}
        onReload={this.handleReload}
      />;
    }

    return this.props.children;
  }
}

interface ErrorFallbackProps {
  error?: Error;
  errorInfo?: ErrorInfo;
  onRetry: () => void;
  onReload: () => void;
}

const ErrorFallback: React.FC<ErrorFallbackProps> = ({
  error,
  errorInfo,
  onRetry,
  onReload,
}) => {
  const theme = useTheme();

  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '400px',
        p: 3,
      }}
    >
      <Paper
        elevation={4}
        sx={{
          p: 4,
          maxWidth: 600,
          width: '100%',
          textAlign: 'center',
          background: alpha(theme.palette.error.main, 0.05),
          border: `1px solid ${alpha(theme.palette.error.main, 0.2)}`,
        }}
      >
        <ErrorIcon
          sx={{
            fontSize: 64,
            color: theme.palette.error.main,
            mb: 2,
          }}
        />

        <Typography variant="h5" gutterBottom color="error">
          Something went wrong
        </Typography>

        <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
          We're sorry, but something unexpected happened. Please try again or reload the page.
        </Typography>

        <Alert severity="error" sx={{ mb: 3, textAlign: 'left' }}>
          <AlertTitle>Error Details</AlertTitle>
          <Typography variant="body2" component="div">
            <strong>Message:</strong> {error?.message || 'Unknown error'}
          </Typography>
          {process.env.NODE_ENV === 'development' && errorInfo && (
            <details style={{ marginTop: 8 }}>
              <summary style={{ cursor: 'pointer', fontWeight: 'bold' }}>
                Stack Trace (Development Only)
              </summary>
              <pre style={{
                marginTop: 8,
                padding: 8,
                backgroundColor: alpha(theme.palette.common.black, 0.1),
                borderRadius: 4,
                fontSize: '0.75rem',
                overflow: 'auto',
                whiteSpace: 'pre-wrap',
              }}>
                {errorInfo.componentStack}
              </pre>
            </details>
          )}
        </Alert>

        <Box sx={{ display: 'flex', gap: 2, justifyContent: 'center', flexWrap: 'wrap' }}>
          <Button
            variant="contained"
            startIcon={<RefreshIcon />}
            onClick={onRetry}
            sx={{ minWidth: 120 }}
          >
            Try Again
          </Button>
          <Button
            variant="outlined"
            onClick={onReload}
            sx={{ minWidth: 120 }}
          >
            Reload Page
          </Button>
        </Box>

        <Typography variant="caption" color="text.secondary" sx={{ mt: 2, display: 'block' }}>
          If this problem persists, please contact support with the error details above.
        </Typography>
      </Paper>
    </Box>
  );
};

export default ErrorBoundary;