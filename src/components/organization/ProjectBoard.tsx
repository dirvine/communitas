import React, { useState, useEffect, useCallback } from 'react';
import {
  Box,
  Typography,
  Stack,
  IconButton,
  Avatar,
  AvatarGroup,
  Chip,
  Button,
  TextField,
  InputAdornment,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  InputLabel,
  Select,
  SelectChangeEvent,
  Tooltip,
  Badge,
  Fade,
  Zoom,
  alpha,
} from '@mui/material';
import {
  Add as AddIcon,
  Search as SearchIcon,
  FilterList as FilterIcon,
  MoreVert as MoreIcon,
  Flag as FlagIcon,
  CalendarToday as CalendarIcon,
  ChatBubble as CommentIcon,
  AttachFile as AttachIcon,
  PlayArrow as PlayIcon,
  CheckCircle as DoneIcon,
  Cancel as CancelIcon,
  Menu as MenuIcon,
} from '@mui/icons-material';
import { DragDropContext, Droppable, Draggable } from '@hello-pangea/dnd';
import { GlassCard } from '../ui/GlassCard';
import { projectService } from '../../services/projectService';
import type {
  Project,
  Issue,
  IssueStatus,
  IssuePriority,
  issueStatusColors,
  issuePriorityColors,
  CreateIssueRequest,
} from '../../types/projects';

interface ProjectBoardProps {
  projectId: string;
  currentUserId: string;
}

const statusConfig: Record<IssueStatus, { label: string; icon: React.ReactNode; color: string }> = {
  backlog: {
    label: 'Backlog',
    icon: <MenuIcon />,
    color: '#94a3b8',
  },
  todo: {
    label: 'To Do',
    icon: <PlayIcon />,
    color: '#3b82f6',
  },
  'in-progress': {
    label: 'In Progress',
    icon: <PlayIcon sx={{ color: '#f59e0b' }} />,
    color: '#f59e0b',
  },
  done: {
    label: 'Done',
    icon: <DoneIcon sx={{ color: '#10b981' }} />,
    color: '#10b981',
  },
  canceled: {
    label: 'Canceled',
    icon: <CancelIcon />,
    color: '#6b7280',
  },
};

const priorityConfig: Record<IssuePriority, { label: string; color: string }> = {
  urgent: { label: 'Urgent', color: '#dc2626' },
  high: { label: 'High', color: '#ea580c' },
  medium: { label: 'Medium', color: '#f59e0b' },
  low: { label: 'Low', color: '#64748b' },
};

export const ProjectBoard: React.FC<ProjectBoardProps> = ({
  projectId,
  currentUserId,
}) => {
  const [project, setProject] = useState<Project | null>(null);
  const [columns, setColumns] = useState<Record<IssueStatus, Issue[]>>({
    backlog: [],
    todo: [],
    'in-progress': [],
    done: [],
    canceled: [],
  });
  const [searchQuery, setSearchQuery] = useState('');
  const [filterPriority, setFilterPriority] = useState<IssuePriority | 'all'>('all');
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [newIssueColumn, setNewIssueColumn] = useState<IssueStatus>('backlog');
  const [selectedIssue, setSelectedIssue] = useState<Issue | null>(null);
  const [detailDialogOpen, setDetailDialogOpen] = useState(false);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);

  // Form state for new issue
  const [newIssueTitle, setNewIssueTitle] = useState('');
  const [newIssueDescription, setNewIssueDescription] = useState('');
  const [newIssuePriority, setNewIssuePriority] = useState<IssuePriority>('medium');

  // Load project and issues
  useEffect(() => {
    loadProject();
    loadIssues();
  }, [projectId]);

  const loadProject = async () => {
    try {
      const proj = await projectService.getProject(projectId);
      setProject(proj);
    } catch (error) {
      console.error('Failed to load project:', error);
    }
  };

  const loadIssues = async () => {
    try {
      const kanban = await projectService.getKanbanBoard(projectId);
      setColumns(kanban);
    } catch (error) {
      console.error('Failed to load issues:', error);
    }
  };

  const handleCreateIssue = async () => {
    if (!newIssueTitle.trim()) return;

    try {
      const request: CreateIssueRequest = {
        project_id: projectId,
        title: newIssueTitle,
        description: newIssueDescription || undefined,
        priority: newIssuePriority,
        reporter_id: currentUserId,
      };

      const issue = await projectService.createIssue(request);

      // Add to appropriate column
      setColumns((prev) => ({
        ...prev,
        [newIssueColumn]: [...prev[newIssueColumn], issue],
      }));

      // Reset form
      setNewIssueTitle('');
      setNewIssueDescription('');
      setNewIssuePriority('medium');
      setCreateDialogOpen(false);
    } catch (error) {
      console.error('Failed to create issue:', error);
    }
  };

  const handleDragEnd = async (result: any) => {
    const { source, destination, draggableId } = result;

    if (!destination) return;
    if (
      source.droppableId === destination.droppableId &&
      source.index === destination.index
    ) {
      return;
    }

    const sourceStatus = source.droppableId as IssueStatus;
    const destStatus = destination.droppableId as IssueStatus;

    // Optimistic update
    const sourceColumn = Array.from(columns[sourceStatus]);
    const destColumn =
      sourceStatus === destStatus
        ? sourceColumn
        : Array.from(columns[destStatus]);

    const [movedIssue] = sourceColumn.splice(source.index, 1);
    destColumn.splice(destination.index, 0, movedIssue);

    setColumns({
      ...columns,
      [sourceStatus]: sourceColumn,
      [destStatus]: destColumn,
    });

    // Update backend
    try {
      if (sourceStatus !== destStatus) {
        await projectService.updateStatus(draggableId, destStatus);
      }
    } catch (error) {
      console.error('Failed to update issue status:', error);
      // Revert on error
      loadIssues();
    }
  };

  const filteredIssues = (issues: Issue[]) => {
    return issues.filter((issue) => {
      const matchesSearch =
        issue.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
        issue.description?.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesPriority =
        filterPriority === 'all' || issue.priority === filterPriority;
      return matchesSearch && matchesPriority;
    });
  };

  const IssueCard: React.FC<{ issue: Issue; index: number }> = ({ issue, index }) => (
    <Draggable draggableId={issue.id} index={index}>
      {(provided, snapshot) => (
        <GlassCard
          ref={provided.innerRef}
          {...provided.draggableProps}
          {...provided.dragHandleProps}
          variant="light"
          blur={15}
          sx={{
            mb: 2,
            cursor: 'grab',
            '&:active': { cursor: 'grabbing' },
            transform: snapshot.isDragging ? 'rotate(2deg)' : 'none',
            transition: 'all 0.2s ease',
          }}
          onClick={() => {
            setSelectedIssue(issue);
            setDetailDialogOpen(true);
          }}
        >
          <Stack spacing={1.5}>
            {/* Header */}
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Chip
                icon={<FlagIcon />}
                label={priorityConfig[issue.priority].label}
                size="small"
                sx={{
                  bgcolor: alpha(priorityConfig[issue.priority].color, 0.15),
                  color: priorityConfig[issue.priority].color,
                  fontWeight: 600,
                  fontSize: '0.7rem',
                }}
              />
              <IconButton size="small" onClick={(e) => {
                e.stopPropagation();
                setMenuAnchor(e.currentTarget);
                setSelectedIssue(issue);
              }}>
                <MoreIcon fontSize="small" />
              </IconButton>
            </Stack>

            {/* Title */}
            <Typography variant="body2" fontWeight={600} sx={{ lineHeight: 1.4 }}>
              {issue.title}
            </Typography>

            {/* Description Preview */}
            {issue.description && (
              <Typography
                variant="caption"
                color="text.secondary"
                sx={{
                  display: '-webkit-box',
                  WebkitLineClamp: 2,
                  WebkitBoxOrient: 'vertical',
                  overflow: 'hidden',
                  lineHeight: 1.3,
                }}
              >
                {issue.description}
              </Typography>
            )}

            {/* Footer */}
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Stack direction="row" spacing={0.5} alignItems="center">
                <CommentIcon sx={{ fontSize: 14, opacity: 0.6 }} />
                <Typography variant="caption" color="text.secondary">
                  0
                </Typography>
                <AttachIcon sx={{ fontSize: 14, opacity: 0.6, ml: 1 }} />
                <Typography variant="caption" color="text.secondary">
                  0
                </Typography>
              </Stack>

              {issue.assignee_id && (
                <Avatar sx={{ width: 24, height: 24 }}>
                  {issue.assignee_id[0].toUpperCase()}
                </Avatar>
              )}
            </Stack>
          </Stack>
        </GlassCard>
      )}
    </Draggable>
  );

  const ColumnView: React.FC<{ status: IssueStatus }> = ({ status }) => {
    const issues = filteredIssues(columns[status]);
    const config = statusConfig[status];

    return (
      <Box sx={{ minWidth: 320, maxWidth: 380, flex: '1 1 320px' }}>
        <GlassCard variant="dark" blur={20} hover={false} sx={{ height: '100%' }}>
          {/* Column Header */}
          <Stack
            direction="row"
            alignItems="center"
            justifyContent="space-between"
            sx={{ mb: 2, pb: 2, borderBottom: `2px solid ${config.color}` }}
          >
            <Stack direction="row" spacing={1} alignItems="center">
              {config.icon}
              <Typography variant="subtitle2" fontWeight={700}>
                {config.label}
              </Typography>
              <Chip label={issues.length} size="small" />
            </Stack>
            <IconButton
              size="small"
              onClick={() => {
                setNewIssueColumn(status);
                setCreateDialogOpen(true);
              }}
            >
              <AddIcon fontSize="small" />
            </IconButton>
          </Stack>

          {/* Droppable Area */}
          <Droppable droppableId={status}>
            {(provided, snapshot) => (
              <Box
                ref={provided.innerRef}
                {...provided.droppableProps}
                sx={{
                  minHeight: 200,
                  bgcolor: snapshot.isDraggingOver
                    ? alpha(config.color, 0.08)
                    : 'transparent',
                  borderRadius: 2,
                  transition: 'background-color 0.2s',
                  p: 1,
                }}
              >
                {issues.map((issue, index) => (
                  <IssueCard key={issue.id} issue={issue} index={index} />
                ))}
                {provided.placeholder}
              </Box>
            )}
          </Droppable>
        </GlassCard>
      </Box>
    );
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <GlassCard
        variant="light"
        blur={20}
        hover={false}
        sx={{ px: 3, py: 2, mb: 3, borderRadius: 3 }}
      >
        <Stack direction="row" alignItems="center" justifyContent="space-between">
          <Stack direction="row" spacing={2} alignItems="center">
            <Box
              sx={{
                width: 48,
                height: 48,
                borderRadius: 2,
                background: project?.color || 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                fontSize: '1.5rem',
              }}
            >
              {project?.icon || '📋'}
            </Box>
            <Box>
              <Typography variant="h6" fontWeight={700}>
                {project?.name || 'Loading...'}
              </Typography>
              {project?.description && (
                <Typography variant="caption" color="text.secondary">
                  {project.description}
                </Typography>
              )}
            </Box>
          </Stack>

          <Stack direction="row" spacing={2}>
            <TextField
              size="small"
              placeholder="Search issues..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              sx={{ width: 250 }}
              InputProps={{
                startAdornment: (
                  <InputAdornment position="start">
                    <SearchIcon fontSize="small" />
                  </InputAdornment>
                ),
              }}
            />
            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>Priority</InputLabel>
              <Select
                value={filterPriority}
                label="Priority"
                onChange={(e) => setFilterPriority(e.target.value as any)}
              >
                <MenuItem value="all">All</MenuItem>
                <MenuItem value="urgent">Urgent</MenuItem>
                <MenuItem value="high">High</MenuItem>
                <MenuItem value="medium">Medium</MenuItem>
                <MenuItem value="low">Low</MenuItem>
              </Select>
            </FormControl>
            <Button
              variant="contained"
              startIcon={<AddIcon />}
              onClick={() => setCreateDialogOpen(true)}
              sx={{
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                '&:hover': {
                  background: 'linear-gradient(135deg, #764ba2 0%, #667eea 100%)',
                },
              }}
            >
              New Issue
            </Button>
          </Stack>
        </Stack>
      </GlassCard>

      {/* Kanban Board */}
      <Box sx={{ flex: 1, overflow: 'auto', px: 2 }}>
        <DragDropContext onDragEnd={handleDragEnd}>
          <Stack direction="row" spacing={3} sx={{ pb: 3 }}>
            {(['backlog', 'todo', 'in-progress', 'done', 'canceled'] as IssueStatus[]).map(
              (status) => (
                <ColumnView key={status} status={status} />
              )
            )}
          </Stack>
        </DragDropContext>
      </Box>

      {/* Create Issue Dialog */}
      <Dialog
        open={createDialogOpen}
        onClose={() => setCreateDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Create New Issue</DialogTitle>
        <DialogContent>
          <Stack spacing={3} sx={{ mt: 1 }}>
            <TextField
              label="Title"
              fullWidth
              value={newIssueTitle}
              onChange={(e) => setNewIssueTitle(e.target.value)}
              autoFocus
            />
            <TextField
              label="Description"
              fullWidth
              multiline
              rows={4}
              value={newIssueDescription}
              onChange={(e) => setNewIssueDescription(e.target.value)}
            />
            <FormControl fullWidth>
              <InputLabel>Priority</InputLabel>
              <Select
                value={newIssuePriority}
                label="Priority"
                onChange={(e) => setNewIssuePriority(e.target.value as IssuePriority)}
              >
                <MenuItem value="low">Low</MenuItem>
                <MenuItem value="medium">Medium</MenuItem>
                <MenuItem value="high">High</MenuItem>
                <MenuItem value="urgent">Urgent</MenuItem>
              </Select>
            </FormControl>
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setCreateDialogOpen(false)}>Cancel</Button>
          <Button onClick={handleCreateIssue} variant="contained">
            Create Issue
          </Button>
        </DialogActions>
      </Dialog>

      {/* Issue Details Dialog (placeholder) */}
      <Dialog
        open={detailDialogOpen}
        onClose={() => setDetailDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>{selectedIssue?.title}</DialogTitle>
        <DialogContent>
          <Typography variant="body2" color="text.secondary">
            {selectedIssue?.description || 'No description'}
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDetailDialogOpen(false)}>Close</Button>
        </DialogActions>
      </Dialog>

      {/* Context Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => setMenuAnchor(null)}>Edit</MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>Assign</MenuItem>
        <MenuItem onClick={() => setMenuAnchor(null)}>Delete</MenuItem>
      </Menu>
    </Box>
  );
};
