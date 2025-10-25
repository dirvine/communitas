import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Typography,
  Alert
} from '@mui/material'
import { Warning } from '@mui/icons-material'

interface RemoveMemberConfirmDialogProps {
  open: boolean
  memberName: string
  onConfirm: () => void
  onCancel: () => void
}

export function RemoveMemberConfirmDialog({
  open,
  memberName,
  onConfirm,
  onCancel
}: RemoveMemberConfirmDialogProps) {
  return (
    <Dialog open={open} onClose={onCancel} maxWidth="sm">
      <DialogTitle>Remove Member</DialogTitle>
      <DialogContent>
        <Alert severity="warning" icon={<Warning />} sx={{ mb: 2 }}>
          This action cannot be undone
        </Alert>
        <Typography>
          Are you sure you want to remove <strong>{memberName}</strong> from this entity?
        </Typography>
        <Typography variant="body2" color="textSecondary" sx={{ mt: 1 }}>
          They will lose access to all shared resources and conversations.
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={onCancel}>
          Cancel
        </Button>
        <Button 
          onClick={onConfirm} 
          color="error" 
          variant="contained"
        >
          Remove
        </Button>
      </DialogActions>
    </Dialog>
  )
}
