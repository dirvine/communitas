/**
 * MarkdownPreview - Rich markdown rendering component
 *
 * Features:
 * - GitHub-flavored markdown support
 * - Syntax highlighting for code blocks
 * - Responsive images and tables
 * - Custom styling for headings, links, lists
 * - Print-friendly layout
 */

import { Box, Paper, styled, Typography, useTheme } from '@mui/material';
import { alpha } from '@mui/material/styles';
import React from 'react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vs, vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import remarkGfm from 'remark-gfm';

interface MarkdownPreviewProps {
  /** Markdown content to render */
  content: string;
  /** Optional custom styling */
  sx?: any;
  /** Enable print mode with optimized styling */
  printMode?: boolean;
}

// Styled container for markdown content
const MarkdownContainer = styled(Paper)(({ theme }) => ({
  padding: theme.spacing(4),
  maxWidth: '900px',
  margin: '0 auto',
  background: alpha(theme.palette.background.paper, 0.9),
  backdropFilter: 'blur(10px)',

  // Typography
  '& h1': {
    fontSize: '2.5rem',
    fontWeight: 700,
    marginTop: theme.spacing(4),
    marginBottom: theme.spacing(2),
    paddingBottom: theme.spacing(1),
    borderBottom: `2px solid ${theme.palette.divider}`,
    color: theme.palette.text.primary,
  },
  '& h2': {
    fontSize: '2rem',
    fontWeight: 600,
    marginTop: theme.spacing(3),
    marginBottom: theme.spacing(2),
    paddingBottom: theme.spacing(0.5),
    borderBottom: `1px solid ${theme.palette.divider}`,
    color: theme.palette.text.primary,
  },
  '& h3': {
    fontSize: '1.5rem',
    fontWeight: 600,
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(1),
    color: theme.palette.text.primary,
  },
  '& h4, & h5, & h6': {
    fontSize: '1.25rem',
    fontWeight: 500,
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(1),
    color: theme.palette.text.secondary,
  },

  // Paragraphs
  '& p': {
    fontSize: '1rem',
    lineHeight: 1.7,
    marginBottom: theme.spacing(2),
    color: theme.palette.text.primary,
  },

  // Links
  '& a': {
    color: theme.palette.primary.main,
    textDecoration: 'none',
    borderBottom: `1px solid ${alpha(theme.palette.primary.main, 0.3)}`,
    transition: 'all 0.2s ease',
    '&:hover': {
      borderBottomColor: theme.palette.primary.main,
      color: theme.palette.primary.dark,
    },
  },

  // Lists
  '& ul, & ol': {
    marginBottom: theme.spacing(2),
    paddingLeft: theme.spacing(4),
  },
  '& li': {
    marginBottom: theme.spacing(1),
    lineHeight: 1.7,
  },
  '& ul ul, & ol ol, & ul ol, & ol ul': {
    marginTop: theme.spacing(1),
    marginBottom: theme.spacing(1),
  },

  // Blockquotes
  '& blockquote': {
    borderLeft: `4px solid ${theme.palette.primary.main}`,
    paddingLeft: theme.spacing(2),
    marginLeft: 0,
    marginRight: 0,
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(2),
    fontStyle: 'italic',
    color: theme.palette.text.secondary,
    background: alpha(theme.palette.primary.main, 0.05),
    padding: theme.spacing(2),
    borderRadius: theme.shape.borderRadius,
  },

  // Code blocks
  '& pre': {
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(2),
    borderRadius: theme.shape.borderRadius,
    overflow: 'auto',
  },
  '& code': {
    fontSize: '0.9em',
    fontFamily: 'Consolas, Monaco, "Courier New", monospace',
  },
  '& :not(pre) > code': {
    background: alpha(theme.palette.text.primary, 0.1),
    padding: '2px 6px',
    borderRadius: 3,
    color: theme.palette.secondary.main,
  },

  // Tables
  '& table': {
    width: '100%',
    borderCollapse: 'collapse',
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(2),
    overflow: 'hidden',
    borderRadius: theme.shape.borderRadius,
  },
  '& th, & td': {
    padding: theme.spacing(1.5),
    textAlign: 'left',
    borderBottom: `1px solid ${theme.palette.divider}`,
  },
  '& th': {
    background: alpha(theme.palette.primary.main, 0.1),
    fontWeight: 600,
    color: theme.palette.text.primary,
  },
  '& tr:hover': {
    background: alpha(theme.palette.action.hover, 0.05),
  },

  // Horizontal rules
  '& hr': {
    border: 'none',
    borderTop: `2px solid ${theme.palette.divider}`,
    marginTop: theme.spacing(3),
    marginBottom: theme.spacing(3),
  },

  // Images
  '& img': {
    maxWidth: '100%',
    height: 'auto',
    borderRadius: theme.shape.borderRadius,
    marginTop: theme.spacing(2),
    marginBottom: theme.spacing(2),
    boxShadow: theme.shadows[2],
  },

  // Task lists (GitHub-flavored)
  '& .task-list-item': {
    listStyle: 'none',
  },
  '& .task-list-item input': {
    marginRight: theme.spacing(1),
  },

  // Print mode
  '@media print': {
    padding: theme.spacing(2),
    boxShadow: 'none',
    background: 'white',
    '& a': {
      color: 'black',
      textDecoration: 'underline',
    },
  },
}));

export const MarkdownPreview: React.FC<MarkdownPreviewProps> = ({
  content,
  sx,
  printMode = false,
}) => {
  const theme = useTheme();
  const isDarkMode = theme.palette.mode === 'dark';

  return (
    <MarkdownContainer
      elevation={printMode ? 0 : 2}
      sx={{
        ...(printMode && {
          boxShadow: 'none',
          background: 'white',
        }),
        ...sx,
      }}
    >
      {content ? (
        <ReactMarkdown
          remarkPlugins={[remarkGfm]}
          components={{
            // Custom code block rendering with syntax highlighting
            code({ node, className, children, ...props }: any) {
              const match = /language-(\w+)/.exec(className || '');
              const language = match ? match[1] : 'text';
              const inline = !match;

              return !inline ? (
                <SyntaxHighlighter
                  style={isDarkMode ? vscDarkPlus : vs}
                  language={language}
                  PreTag="div"
                  customStyle={{
                    margin: 0,
                    borderRadius: theme.shape.borderRadius,
                  }}
                >
                  {String(children).replace(/\n$/, '')}
                </SyntaxHighlighter>
              ) : (
                <code className={className} {...props}>
                  {children}
                </code>
              );
            },

            // Ensure links open in new tab for external URLs
            a({ node, href, children, ...props }: any) {
              const isExternal = href?.startsWith('http');
              return (
                <a
                  href={href}
                  target={isExternal ? '_blank' : undefined}
                  rel={isExternal ? 'noopener noreferrer' : undefined}
                  {...props}
                >
                  {children}
                </a>
              );
            },

            // Custom table rendering
            table({ node, children, ...props }: any) {
              return (
                <Box sx={{ overflowX: 'auto', mb: 2 }}>
                  <table {...props}>{children}</table>
                </Box>
              );
            },
          }}
        >
          {content}
        </ReactMarkdown>
      ) : (
        <Box sx={{ textAlign: 'center', py: 8 }}>
          <Typography variant="h6" color="text.secondary">
            No content to preview
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
            Start editing to see your markdown rendered here
          </Typography>
        </Box>
      )}
    </MarkdownContainer>
  );
};
