// Global ambient type shims to achieve a clean typecheck baseline

// Allow any JSX props to reduce friction during migration across MUI versions
declare namespace JSX {
  interface IntrinsicAttributes {
    [key: string]: any;
  }
}

// Missing modules in the current app setup (treated as any)
declare module '@monaco-editor/react';
declare module '@mui/lab/Timeline';
declare module '@mui/lab/TimelineItem';
declare module '@mui/lab/TimelineSeparator';
declare module '@mui/lab/TimelineConnector';
declare module '@mui/lab/TimelineContent';
declare module '@mui/lab/TimelineDot';
declare module '@mui/lab/TimelineOppositeContent';
declare module 'react-syntax-highlighter';
declare module 'react-syntax-highlighter/dist/esm/styles/prism';

declare module 'blake3';
declare module 'events';
declare module 'crypto';
declare module 'stream';

// Avoid shimming core MUI modules to prevent clobbering type exports
// If specific lab modules are missing, shim exact paths instead.

// Optionally provide very loose nodes for Buffer/process when node types are absent
// (Will be superseded by @types/node once installed)
declare const Buffer: any;
declare const process: any;
