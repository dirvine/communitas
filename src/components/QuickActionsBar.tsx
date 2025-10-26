import {
    Brush as BrushIcon, Business as BusinessIcon, Chat as ChatIcon, Close as CloseIcon, CloudUpload as CloudUploadIcon, CreateNewFolder as CreateFolderIcon, Delete as DeleteIcon, Edit as EditIcon,
    FileCopy as FileCopyIcon, GroupAdd as GroupAddIcon, Link as LinkIcon, Menu as MenuIcon, Notifications as NotificationsIcon, PersonAdd as PersonAddIcon, Phone as PhoneIcon, QrCode as QrCodeIcon, Search as SearchIcon, Settings as SettingsIcon, Share as ShareIcon, Storage as StorageIcon, Videocam as VideocamIcon, VpnKey as VpnKeyIcon
} from '@mui/icons-material';
import {
    Badge, Divider, IconButton, ListItemIcon,
    ListItemText, Menu,
    MenuItem, SpeedDial,
    SpeedDialAction,
    SpeedDialIcon,
    Tooltip
} from '@mui/material';
import React, { useState } from 'react';
import { useTheme } from './theme/ThemeProvider';

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
  const { mode: _mode, toggleMode: _toggleMode, setColorPreset: _setColorPreset } = useTheme();

  const handleAction = (action: string, data?: any) => {
    setOpen(false);
    onAction(action, data);
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
        return { ...baseStyles, bottom: 24, right: 24 };
      case 'bottom-left':
        return { ...baseStyles, bottom: 24, left: 24 };
      case 'top-right':
        return { ...baseStyles, top: 80, right: 24 };
      case 'top-left':
        return { ...baseStyles, top: 80, left: 24 };
      default:
        return { ...baseStyles, bottom: 24, right: 24 };
    }
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

      {/* Context Menu */}
      {renderContextMenu()}
    </>
  );
};

// Settings button component to be placed in the header
export const SettingsButton: React.FC<{ onAction: (action: string) => void }> = ({ onAction }) => {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const handleSettingsClick = () => {
    onAction('settings');
    handleClose();
  };

  const handleThemeToggle = () => {
    onAction('toggle_theme');
    handleClose();
  };

  return (
    <>
      <Tooltip title="Settings & Options">
        <IconButton
          onClick={handleClick}
          size="medium"
          sx={{
            ml: 1,
            border: '1px solid',
            borderColor: 'divider',
            '&:hover': {
              backgroundColor: 'action.hover',
            }
          }}
        >
          <MenuIcon />
        </IconButton>
      </Tooltip>

      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={handleClose}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'right',
        }}
        transformOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
        PaperProps={{
          sx: {
            minWidth: 200,
            mt: 1,
          }
        }}
      >
        <MenuItem onClick={handleSettingsClick}>
          <ListItemIcon><SettingsIcon /></ListItemIcon>
          <ListItemText>Settings</ListItemText>
        </MenuItem>

        <MenuItem onClick={handleThemeToggle}>
          <ListItemIcon><BrushIcon /></ListItemIcon>
          <ListItemText>Toggle Theme</ListItemText>
        </MenuItem>

        <Divider />

        <MenuItem onClick={() => { onAction('storage_settings'); handleClose(); }}>
          <ListItemIcon><StorageIcon /></ListItemIcon>
          <ListItemText>Storage</ListItemText>
        </MenuItem>

        <MenuItem onClick={() => { onAction('manage_permissions'); handleClose(); }}>
          <ListItemIcon><VpnKeyIcon /></ListItemIcon>
          <ListItemText>Permissions</ListItemText>
        </MenuItem>
      </Menu>
    </>
  );
};

export default QuickActionsBar;
