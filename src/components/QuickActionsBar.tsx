import React, { useState } from 'react';
import {
  Box,
  SpeedDial,
  SpeedDialAction,
  SpeedDialIcon,
  Fab,
  Stack,
  Tooltip,
  Badge,
  IconButton,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Avatar,
  Typography,
  Paper,
} from '@mui/material';
import {
  Add as AddIcon,
  Close as CloseIcon,
  Phone as PhoneIcon,
  Videocam as VideocamIcon,
  Chat as ChatIcon,
  FolderOpen as FolderIcon,
  Upload as UploadIcon,
  GroupAdd as GroupAddIcon,
  CreateNewFolder as CreateFolderIcon,
  Business as BusinessIcon,
  PersonAdd as PersonAddIcon,
  Share as ShareIcon,
  Settings as SettingsIcon,
  Notifications as NotificationsIcon,
  Search as SearchIcon,
  Edit as EditIcon,
  FileCopy as FileCopyIcon,
  Delete as DeleteIcon,
  CloudUpload as CloudUploadIcon,
  CloudDownload as CloudDownloadIcon,
  Link as LinkIcon,
  QrCode as QrCodeIcon,
  VpnKey as VpnKeyIcon,
  Storage as StorageIcon,
} from '@mui/icons-material';

interface QuickActionsBarProps {
  context: {
    type: 'personal' | 'organization' | 'project' | 'group';
    entity?: any;
  };
  onAction: (action: string, data?: any) => void;
  position?: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left';
  notifications?: number;
}

export const QuickActionsBar: React.FC<QuickActionsBarProps> = ({
  context,
  onAction,
  position = 'bottom-right',
  notifications = 0,
}) => {
  const [open, setOpen] = useState(false);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [activeMenu, setActiveMenu] = useState<string | null>(null);

  const handleAction = (action: string, data?: any) => {
    setOpen(false);
    onAction(action, data);
  };

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>, menu: string) => {
    setMenuAnchor(event.currentTarget);
    setActiveMenu(menu);
  };

  const handleMenuClose = () => {
    setMenuAnchor(null);
    setActiveMenu(null);
  };

  // Define context-specific actions
  const getContextActions = () => {
    const baseActions = [
      { 
        icon: <SearchIcon />, 
        name: 'Search', 
        action: 'search',
        color: 'default' as const,
      },
      { 
        icon: <NotificationsIcon />, 
        name: 'Notifications', 
        action: 'notifications',
        badge: notifications,
        color: 'default' as const,
      },
      { 
        icon: <SettingsIcon />, 
        name: 'Settings', 
        action: 'settings',
        color: 'default' as const,
      },
    ];

    switch (context.type) {
      case 'personal':
        return [
          { 
            icon: <BusinessIcon />, 
            name: 'Create Organization', 
            action: 'create_organization',
            color: 'primary' as const,
          },
          { 
            icon: <PersonAddIcon />, 
            name: 'Add Contact', 
            action: 'add_contact',
            color: 'secondary' as const,
          },
          { 
            icon: <CloudUploadIcon />, 
            name: 'Upload Files', 
            action: 'upload_files',
            color: 'default' as const,
          },
          ...baseActions,
        ];

      case 'organization':
        return [
          { 
            icon: <CreateFolderIcon />, 
            name: 'Create Project', 
            action: 'create_project',
            color: 'primary' as const,
          },
          { 
            icon: <GroupAddIcon />, 
            name: 'Create Group', 
            action: 'create_group',
            color: 'secondary' as const,
          },
          { 
            icon: <PersonAddIcon />, 
            name: 'Invite Members', 
            action: 'invite_members',
            color: 'default' as const,
          },
          { 
            icon: <ShareIcon />, 
            name: 'Share', 
            action: 'share_organization',
            color: 'default' as const,
          },
          ...baseActions,
        ];

      case 'project':
        return [
          { 
            icon: <CloudUploadIcon />, 
            name: 'Upload Documents', 
            action: 'upload_documents',
            color: 'primary' as const,
          },
          { 
            icon: <PersonAddIcon />, 
            name: 'Add Team Member', 
            action: 'add_team_member',
            color: 'secondary' as const,
          },
          { 
            icon: <EditIcon />, 
            name: 'Edit Project', 
            action: 'edit_project',
            color: 'default' as const,
          },
          { 
            icon: <LinkIcon />, 
            name: 'Share Link', 
            action: 'share_link',
            color: 'default' as const,
          },
          ...baseActions,
        ];

      case 'group':
        return [
          { 
            icon: <PhoneIcon />, 
            name: 'Start Voice Call', 
            action: 'start_voice_call',
            color: 'success' as const,
          },
          { 
            icon: <VideocamIcon />, 
            name: 'Start Video Call', 
            action: 'start_video_call',
            color: 'primary' as const,
          },
          { 
            icon: <PersonAddIcon />, 
            name: 'Add Members', 
            action: 'add_members',
            color: 'secondary' as const,
          },
          { 
            icon: <ChatIcon />, 
            name: 'Open Chat', 
            action: 'open_chat',
            color: 'default' as const,
          },
          ...baseActions,
        ];

      default:
        return baseActions;
    }
  };

  const actions = getContextActions() as Array<{
    icon: React.ReactNode;
    name: string;
    action: string;
    color: 'default' | 'primary' | 'secondary' | 'success';
    badge?: number;
  }>;

  // Position styles
  const getPositionStyles = () => {
    const baseStyles = { position: 'fixed' as const, zIndex: 1200 };
    
    switch (position) {
      case 'bottom-right':
        return { ...baseStyles, bottom: 16, right: 16 };
      case 'bottom-left':
        return { ...baseStyles, bottom: 16, left: 16 };
      case 'top-right':
        return { ...baseStyles, top: 80, right: 16 };
      case 'top-left':
        return { ...baseStyles, top: 80, left: 16 };
      default:
        return { ...baseStyles, bottom: 16, right: 16 };
    }
  };

  // Floating Action Buttons for common actions
  const renderFloatingActions = () => {
    const showCallButtons = context.type === 'organization' || 
                           context.type === 'project' || 
                           context.type === 'group' ||
                           context.type === 'personal';

    if (!showCallButtons) return null;

    return (
      <Stack
        direction="column"
        spacing={1}
        sx={{
          position: 'fixed',
          bottom: 100,
          right: position.includes('right') ? 16 : undefined,
          left: position.includes('left') ? 16 : undefined,
          zIndex: 1199,
        }}
      >
        <Tooltip title="Voice Call" placement={position.includes('right') ? 'left' : 'right'}>
          <Fab
            size="small"
            color="success"
            onClick={() => handleAction('start_voice_call')}
            sx={{ 
              boxShadow: 2,
              '&:hover': { transform: 'scale(1.1)' },
              transition: 'transform 0.2s',
            }}
          >
            <PhoneIcon />
          </Fab>
        </Tooltip>
        
        <Tooltip title="Video Call" placement={position.includes('right') ? 'left' : 'right'}>
          <Fab
            size="small"
            color="primary"
            onClick={() => handleAction('start_video_call')}
            sx={{ 
              boxShadow: 2,
              '&:hover': { transform: 'scale(1.1)' },
              transition: 'transform 0.2s',
            }}
          >
            <VideocamIcon />
          </Fab>
        </Tooltip>
        
        <Tooltip title="Storage" placement={position.includes('right') ? 'left' : 'right'}>
          <Fab
            size="small"
            color="info"
            onClick={() => handleAction('storage_settings')}
            sx={{ 
              boxShadow: 2,
              '&:hover': { transform: 'scale(1.1)' },
              transition: 'transform 0.2s',
            }}
          >
            <StorageIcon />
          </Fab>
        </Tooltip>
      </Stack>
    );
  };

  // Context Menu for advanced actions
  const renderContextMenu = () => (
    <Menu
      anchorEl={menuAnchor}
      open={activeMenu === 'advanced'}
      onClose={handleMenuClose}
      PaperProps={{
        sx: { width: 240 },
      }}
    >
      <MenuItem onClick={() => { handleMenuClose(); handleAction('generate_invite_link'); }}>
        <ListItemIcon>
          <LinkIcon fontSize="small" />
        </ListItemIcon>
        <ListItemText>Generate Invite Link</ListItemText>
      </MenuItem>
      
      <MenuItem onClick={() => { handleMenuClose(); handleAction('show_qr_code'); }}>
        <ListItemIcon>
          <QrCodeIcon fontSize="small" />
        </ListItemIcon>
        <ListItemText>Show QR Code</ListItemText>
      </MenuItem>
      
      <MenuItem onClick={() => { handleMenuClose(); handleAction('manage_permissions'); }}>
        <ListItemIcon>
          <VpnKeyIcon fontSize="small" />
        </ListItemIcon>
        <ListItemText>Manage Permissions</ListItemText>
      </MenuItem>
      
      <Divider />
      
      <MenuItem onClick={() => { handleMenuClose(); handleAction('storage_settings'); }}>
        <ListItemIcon>
          <StorageIcon fontSize="small" />
        </ListItemIcon>
        <ListItemText>Storage Settings</ListItemText>
      </MenuItem>
      
      <MenuItem onClick={() => { handleMenuClose(); handleAction('duplicate'); }}>
        <ListItemIcon>
          <FileCopyIcon fontSize="small" />
        </ListItemIcon>
        <ListItemText>Duplicate</ListItemText>
      </MenuItem>
      
      {context.type !== 'personal' && (
        <MenuItem 
          onClick={() => { handleMenuClose(); handleAction('delete'); }}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon>
            <DeleteIcon fontSize="small" color="error" />
          </ListItemIcon>
          <ListItemText>Delete {context.type}</ListItemText>
        </MenuItem>
      )}
    </Menu>
  );

  // Mini floating toolbar for quick actions
  const renderMiniToolbar = () => {
    if (context.type === 'personal') return null;

    return (
      <Paper
        elevation={3}
        sx={{
          position: 'fixed',
          top: 80,
          left: '50%',
          transform: 'translateX(-50%)',
          zIndex: 1200,
          borderRadius: 2,
          overflow: 'hidden',
        }}
      >
        <Stack direction="row" spacing={0}>
          <Tooltip title="Upload">
            <IconButton onClick={() => handleAction('upload')}>
              <CloudUploadIcon />
            </IconButton>
          </Tooltip>
          
          <Divider orientation="vertical" flexItem />
          
          <Tooltip title="Share">
            <IconButton onClick={() => handleAction('share')}>
              <ShareIcon />
            </IconButton>
          </Tooltip>
          
          <Tooltip title="Edit">
            <IconButton onClick={() => handleAction('edit')}>
              <EditIcon />
            </IconButton>
          </Tooltip>
          
          <Divider orientation="vertical" flexItem />
          
          <Tooltip title="More Actions">
            <IconButton onClick={(e) => handleMenuOpen(e, 'advanced')}>
              <AddIcon />
            </IconButton>
          </Tooltip>
        </Stack>
      </Paper>
    );
  };

  return (
    <>
      {/* Main Speed Dial */}
      <SpeedDial
        ariaLabel="Quick Actions"
        sx={getPositionStyles()}
        icon={<SpeedDialIcon openIcon={<CloseIcon />} />}
        open={open}
        onClose={() => setOpen(false)}
        onOpen={() => setOpen(true)}
        FabProps={{
          color: 'primary',
          size: 'large',
          sx: {
            boxShadow: 4,
            '&:hover': { 
              transform: 'scale(1.05)',
              boxShadow: 6,
            },
            transition: 'all 0.3s',
          },
        }}
      >
        {actions.map((action) => (
          <SpeedDialAction
            key={action.action}
            icon={
              action.badge ? (
                <Badge badgeContent={action.badge} color="error">
                  {action.icon}
                </Badge>
              ) : (
                action.icon
              )
            }
            tooltipTitle={action.name}
            tooltipOpen
            onClick={() => handleAction(action.action)}
            FabProps={{
              color: action.color,
              sx: {
                '&:hover': { transform: 'scale(1.1)' },
                transition: 'transform 0.2s',
              },
            }}
          />
        ))}
      </SpeedDial>

      {/* Floating Call Actions */}
      {renderFloatingActions()}

      {/* Mini Toolbar */}
      {renderMiniToolbar()}

      {/* Context Menu */}
      {renderContextMenu()}
    </>
  );
};

export default QuickActionsBar;