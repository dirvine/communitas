import React, { useState, useEffect } from 'react';
import {
  Box,
  Paper,
  Typography,
  Grid,
  Card,
  CardContent,
  CardActions,
  Button,
  Chip,
  LinearProgress,
  Stack,
  Avatar,
  AvatarGroup,
  IconButton,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  FormControl,
  InputLabel,
  Select,
  Tab,
  Tabs,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  Divider,
  Tooltip,
  Badge,
  Fab,
  SpeedDial,
  SpeedDialAction,
  SpeedDialIcon,
  Collapse,
  Alert,
} from '@mui/material';
import {
  Timeline,
  TimelineItem,
  TimelineSeparator,
  TimelineConnector,
  TimelineContent,
  TimelineDot,
  TimelineOppositeContent,
} from '@mui/lab';
import {
  Assignment as TaskIcon,
  Description as FileIcon,
  Group as TeamIcon,
  Timeline as TimelineIcon,
  Settings as SettingsIcon,
  Add as AddIcon,
  Close as CloseIcon,
  Edit as EditIcon,
  Delete as DeleteIcon,
  Share as ShareIcon,
  Phone as PhoneIcon,
  Videocam as VideocamIcon,
  Upload as UploadIcon,
  Download as DownloadIcon,
  Star as StarIcon,
  StarBorder as StarBorderIcon,
  Flag as FlagIcon,
  CheckCircle as CompleteIcon,
  RadioButtonUnchecked as IncompleteIcon,
  Schedule as ScheduleIcon,
  PersonAdd as PersonAddIcon,
  CreateNewFolder as CreateFolderIcon,
  Link as LinkIcon,
  Visibility as ViewIcon,
  CloudUpload as CloudUploadIcon,
  Folder as FolderIcon,
  InsertChart as ChartIcon,
  CalendarToday as CalendarIcon,
  Comment as CommentIcon,
  NotificationsActive as NotificationIcon,
  Security as SecurityIcon,
  Backup as BackupIcon,
  Public as PublicIcon,
  Lock as LockIcon,
  ExpandMore as ExpandMoreIcon,
  ExpandLess as ExpandLessIcon,
} from '@mui/icons-material';
import { motion, AnimatePresence } from 'framer-motion';
import { Project, Member } from '../../types/organization';

interface ProjectViewProps {
  project: Project;
  onAction: (action: string, data?: any) => void;
  onCall: (callType: 'voice' | 'video') => void;
  onMemberSelect: (member: Member) => void;
}

interface Task {
  id: string;
  title: string;
  description: string;
  status: 'todo' | 'in_progress' | 'review' | 'completed';
  priority: 'low' | 'medium' | 'high' | 'critical';
  assignee?: Member;
  due_date?: string;
  created_at: string;
  updated_at: string;
  tags: string[];
  subtasks: SubTask[];
  comments: TaskComment[];
  attachments: TaskAttachment[];
}

interface SubTask {
  id: string;
  title: string;
  completed: boolean;
  assignee?: Member;
}

interface TaskComment {
  id: string;
  author: Member;
  content: string;
  created_at: string;
}

interface TaskAttachment {
  id: string;
  filename: string;
  size: string;
  uploaded_by: Member;
  uploaded_at: string;
}

interface ProjectFile {
  id: string;
  name: string;
  type: 'document' | 'image' | 'video' | 'archive' | 'other';
  size: string;
  uploaded_by: Member;
  uploaded_at: string;
  shared: boolean;
  starred: boolean;
  path: string;
}

interface Milestone {
  id: string;
  title: string;
  description: string;
  due_date: string;
  completed: boolean;
  tasks: string[]; // Task IDs
  completion_percentage: number;
}

const TabPanel: React.FC<{ children?: React.ReactNode; value: number; index: number }> = ({
  children,
  value,
  index,
}) => (
  <div hidden={value !== index} style={{ height: '100%' }}>
    {value === index && <Box sx={{ height: '100%', py: 2 }}>{children}</Box>}
  </div>
);

export const ProjectView: React.FC<ProjectViewProps> = ({
  project,
  onAction,
  onCall,
  onMemberSelect,
}) => {
  const [activeTab, setActiveTab] = useState(0);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [files, setFiles] = useState<ProjectFile[]>([]);
  const [milestones, setMilestones] = useState<Milestone[]>([]);
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [taskDialogOpen, setTaskDialogOpen] = useState(false);
  const [fileDialogOpen, setFileDialogOpen] = useState(false);
  const [speedDialOpen, setSpeedDialOpen] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['overview']));

  useEffect(() => {
    loadProjectData();
  }, [project.id]);

  const loadProjectData = async () => {
    // Mock data - in real app, fetch from backend
    setTasks([
      {
        id: '1',
        title: 'Design System Implementation',
        description: 'Create comprehensive design system with components',
        status: 'in_progress',
        priority: 'high',
        assignee: project.members[0],
        due_date: '2025-01-20',
        created_at: '2025-01-10T10:00:00Z',
        updated_at: '2025-01-11T15:30:00Z',
        tags: ['design', 'frontend'],
        subtasks: [
          { id: 's1', title: 'Color palette', completed: true, assignee: project.members[0] },
          { id: 's2', title: 'Typography system', completed: true },
          { id: 's3', title: 'Component library', completed: false, assignee: project.members[1] },
        ],
        comments: [],
        attachments: [],
      },
      {
        id: '2',
        title: 'API Documentation',
        description: 'Complete API documentation for all endpoints',
        status: 'todo',
        priority: 'medium',
        due_date: '2025-01-25',
        created_at: '2025-01-10T11:00:00Z',
        updated_at: '2025-01-10T11:00:00Z',
        tags: ['documentation', 'api'],
        subtasks: [],
        comments: [],
        attachments: [],
      },
      {
        id: '3',
        title: 'Security Audit',
        description: 'Comprehensive security review and penetration testing',
        status: 'completed',
        priority: 'critical',
        assignee: project.members[1],
        created_at: '2025-01-05T09:00:00Z',
        updated_at: '2025-01-09T16:00:00Z',
        tags: ['security', 'audit'],
        subtasks: [],
        comments: [],
        attachments: [],
      },
    ]);

    setFiles([
      {
        id: '1',
        name: 'Project Requirements.pdf',
        type: 'document',
        size: '2.4 MB',
        uploaded_by: project.members[0],
        uploaded_at: '2025-01-10T10:00:00Z',
        shared: true,
        starred: true,
        path: '/docs/requirements.pdf',
      },
      {
        id: '2',
        name: 'Design Assets.zip',
        type: 'archive',
        size: '45.2 MB',
        uploaded_by: project.members[1],
        uploaded_at: '2025-01-11T14:30:00Z',
        shared: false,
        starred: false,
        path: '/assets/design.zip',
      },
    ]);

    setMilestones([
      {
        id: '1',
        title: 'MVP Release',
        description: 'Minimum viable product with core features',
        due_date: '2025-02-01',
        completed: false,
        tasks: ['1', '2'],
        completion_percentage: 60,
      },
      {
        id: '2',
        title: 'Beta Launch',
        description: 'Public beta with user testing',
        due_date: '2025-03-01',
        completed: false,
        tasks: ['3'],
        completion_percentage: 20,
      },
    ]);
  };

  const handleMenuOpen = (event: React.MouseEvent<HTMLElement>) => {
    setMenuAnchor(event.currentTarget);
  };

  const handleMenuClose = () => {
    setMenuAnchor(null);
  };

  const toggleSection = (section: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(section)) {
      newExpanded.delete(section);
    } else {
      newExpanded.add(section);
    }
    setExpandedSections(newExpanded);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'success';
      case 'on_hold': return 'warning';
      case 'completed': return 'info';
      case 'archived': return 'default';
      default: return 'default';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical': return 'error';
      case 'high': return 'warning';
      case 'medium': return 'info';
      case 'low': return 'default';
      default: return 'default';
    }
  };

  const getTaskStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'success';
      case 'in_progress': return 'primary';
      case 'review': return 'warning';
      case 'todo': return 'default';
      default: return 'default';
    }
  };

  const renderOverviewTab = () => (
    <Grid container spacing={3}>
      {/* Project Header */}
      <Grid item xs={12}>
        <Card elevation={2}>
          <CardContent>
            <Stack direction="row" justifyContent="space-between" alignItems="start" spacing={2}>
              <Box>
                <Typography variant="h4" gutterBottom>
                  {project.name}
                </Typography>
                <Typography variant="body1" color="text.secondary" paragraph>
                  {project.description}
                </Typography>
                <Stack direction="row" spacing={1} sx={{ mb: 2 }}>
                  <Chip 
                    label={project.status} 
                    color={getStatusColor(project.status) as any}
                    size="small"
                  />
                  <Chip 
                    label={project.priority} 
                    color={getPriorityColor(project.priority) as any}
                    size="small"
                  />
                  {project.deadline && (
                    <Chip 
                      icon={<ScheduleIcon />}
                      label={`Due: ${new Date(project.deadline).toLocaleDateString()}`} 
                      size="small"
                      variant="outlined"
                    />
                  )}
                </Stack>
                <AvatarGroup max={4} sx={{ justifyContent: 'flex-start' }}>
                  {project.members.map((member, index) => (
                    <Tooltip key={member.user_id} title={member.display_name}>
                      <Avatar 
                        onClick={() => onMemberSelect(member)}
                        sx={{ cursor: 'pointer' }}
                      >
                        {member.display_name[0]}
                      </Avatar>
                    </Tooltip>
                  ))}
                </AvatarGroup>
              </Box>
              <Stack spacing={1}>
                <Button
                  variant="contained"
                  color="success"
                  startIcon={<PhoneIcon />}
                  onClick={() => onCall('voice')}
                >
                  Voice Call
                </Button>
                <Button
                  variant="contained"
                  startIcon={<VideocamIcon />}
                  onClick={() => onCall('video')}
                >
                  Video Call
                </Button>
                <IconButton onClick={handleMenuOpen}>
                  <SettingsIcon />
                </IconButton>
              </Stack>
            </Stack>
          </CardContent>
        </Card>
      </Grid>

      {/* Project Statistics */}
      <Grid item xs={12} md={6} lg={3}>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
        >
          <Card>
            <CardContent>
              <Stack direction="row" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="text.secondary" gutterBottom variant="overline">
                    Total Tasks
                  </Typography>
                  <Typography variant="h4">
                    {tasks.length}
                  </Typography>
                </Box>
                <TaskIcon color="primary" sx={{ fontSize: 40 }} />
              </Stack>
              <Box sx={{ mt: 1 }}>
                <Typography variant="body2" color="text.secondary">
                  {tasks.filter(t => t.status === 'completed').length} completed
                </Typography>
                <LinearProgress 
                  variant="determinate" 
                  value={(tasks.filter(t => t.status === 'completed').length / tasks.length) * 100}
                  sx={{ mt: 1 }}
                />
              </Box>
            </CardContent>
          </Card>
        </motion.div>
      </Grid>

      <Grid item xs={12} md={6} lg={3}>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
        >
          <Card>
            <CardContent>
              <Stack direction="row" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="text.secondary" gutterBottom variant="overline">
                    Team Members
                  </Typography>
                  <Typography variant="h4">
                    {project.members.length}
                  </Typography>
                </Box>
                <TeamIcon color="secondary" sx={{ fontSize: 40 }} />
              </Stack>
              <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                Active collaborators
              </Typography>
            </CardContent>
          </Card>
        </motion.div>
      </Grid>

      <Grid item xs={12} md={6} lg={3}>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
        >
          <Card>
            <CardContent>
              <Stack direction="row" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="text.secondary" gutterBottom variant="overline">
                    Files & Documents
                  </Typography>
                  <Typography variant="h4">
                    {files.length}
                  </Typography>
                </Box>
                <FileIcon color="info" sx={{ fontSize: 40 }} />
              </Stack>
              <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                {files.filter(f => f.shared).length} shared
              </Typography>
            </CardContent>
          </Card>
        </motion.div>
      </Grid>

      <Grid item xs={12} md={6} lg={3}>
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
        >
          <Card>
            <CardContent>
              <Stack direction="row" alignItems="center" justifyContent="space-between">
                <Box>
                  <Typography color="text.secondary" gutterBottom variant="overline">
                    Storage Used
                  </Typography>
                  <Typography variant="h4">
                    {project.storage_quota.used_gb.toFixed(1)}GB
                  </Typography>
                </Box>
                <FolderIcon color="warning" sx={{ fontSize: 40 }} />
              </Stack>
              <Box sx={{ mt: 1 }}>
                <Typography variant="body2" color="text.secondary">
                  of {project.storage_quota.allocated_gb}GB allocated
                </Typography>
                <LinearProgress 
                  variant="determinate" 
                  value={(project.storage_quota.used_gb / project.storage_quota.allocated_gb) * 100}
                  sx={{ mt: 1 }}
                />
              </Box>
            </CardContent>
          </Card>
        </motion.div>
      </Grid>

      {/* Milestones */}
      <Grid item xs={12} md={6}>
        <Card>
          <CardContent>
            <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
              <Typography variant="h6">Project Milestones</Typography>
              <Button size="small" startIcon={<AddIcon />} onClick={() => onAction('create_milestone')}>
                Add Milestone
              </Button>
            </Stack>
            <Stack spacing={2}>
              {milestones.map((milestone) => (
                <Card key={milestone.id} variant="outlined">
                  <CardContent sx={{ py: 2 }}>
                    <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 1 }}>
                      <FlagIcon color={milestone.completed ? 'success' : 'warning'} />
                      <Typography variant="subtitle1">{milestone.title}</Typography>
                      {milestone.completed && <Chip label="Completed" color="success" size="small" />}
                    </Stack>
                    <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                      {milestone.description}
                    </Typography>
                    <Box sx={{ mb: 1 }}>
                      <Stack direction="row" justifyContent="space-between" sx={{ mb: 0.5 }}>
                        <Typography variant="caption">Progress</Typography>
                        <Typography variant="caption">{milestone.completion_percentage}%</Typography>
                      </Stack>
                      <LinearProgress 
                        variant="determinate" 
                        value={milestone.completion_percentage}
                        color={milestone.completed ? 'success' : 'primary'}
                      />
                    </Box>
                    <Typography variant="caption" color="text.secondary">
                      Due: {new Date(milestone.due_date).toLocaleDateString()}
                    </Typography>
                  </CardContent>
                </Card>
              ))}
            </Stack>
          </CardContent>
        </Card>
      </Grid>

      {/* Recent Activity */}
      <Grid item xs={12} md={6}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>Recent Activity</Typography>
            <Timeline>
              <TimelineItem>
                <TimelineOppositeContent sx={{ m: 'auto 0' }} variant="body2" color="text.secondary">
                  2 hours ago
                </TimelineOppositeContent>
                <TimelineSeparator>
                  <TimelineDot color="primary">
                    <TaskIcon />
                  </TimelineDot>
                  <TimelineConnector />
                </TimelineSeparator>
                <TimelineContent sx={{ py: '12px', px: 2 }}>
                  <Typography variant="subtitle2" component="span">
                    Task Updated
                  </Typography>
                  <Typography>Design System Implementation marked as in progress</Typography>
                </TimelineContent>
              </TimelineItem>
              <TimelineItem>
                <TimelineOppositeContent sx={{ m: 'auto 0' }} variant="body2" color="text.secondary">
                  1 day ago
                </TimelineOppositeContent>
                <TimelineSeparator>
                  <TimelineDot color="secondary">
                    <FileIcon />
                  </TimelineDot>
                  <TimelineConnector />
                </TimelineSeparator>
                <TimelineContent sx={{ py: '12px', px: 2 }}>
                  <Typography variant="subtitle2" component="span">
                    File Uploaded
                  </Typography>
                  <Typography>Design Assets.zip added to project</Typography>
                </TimelineContent>
              </TimelineItem>
              <TimelineItem>
                <TimelineOppositeContent sx={{ m: 'auto 0' }} variant="body2" color="text.secondary">
                  3 days ago
                </TimelineOppositeContent>
                <TimelineSeparator>
                  <TimelineDot color="success">
                    <CompleteIcon />
                  </TimelineDot>
                </TimelineSeparator>
                <TimelineContent sx={{ py: '12px', px: 2 }}>
                  <Typography variant="subtitle2" component="span">
                    Task Completed
                  </Typography>
                  <Typography>Security Audit successfully completed</Typography>
                </TimelineContent>
              </TimelineItem>
            </Timeline>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );

  const renderTasksTab = () => (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 3 }}>
        <Typography variant="h5">Project Tasks</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setTaskDialogOpen(true)}
        >
          Create Task
        </Button>
      </Stack>

      {/* Task Kanban Board */}
      <Grid container spacing={2}>
        {[
          { status: 'todo', title: 'To Do', color: 'default' },
          { status: 'in_progress', title: 'In Progress', color: 'primary' },
          { status: 'review', title: 'Review', color: 'warning' },
          { status: 'completed', title: 'Completed', color: 'success' },
        ].map((column) => (
          <Grid item xs={12} sm={6} md={3} key={column.status}>
            <Card variant="outlined" sx={{ minHeight: 400 }}>
              <CardContent>
                <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
                  <Typography variant="h6">{column.title}</Typography>
                  <Chip 
                    label={tasks.filter(t => t.status === column.status).length}
                    color={column.color as any}
                    size="small"
                  />
                </Stack>
                
                <Stack spacing={2}>
                  {tasks
                    .filter(task => task.status === column.status)
                    .map((task) => (
                      <motion.div
                        key={task.id}
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                      >
                        <Card 
                          variant="outlined"
                          sx={{ 
                            cursor: 'pointer',
                            '&:hover': { boxShadow: 2 }
                          }}
                          onClick={() => setSelectedTask(task)}
                        >
                          <CardContent sx={{ pb: '12px !important' }}>
                            <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 1 }}>
                              <Chip 
                                label={task.priority} 
                                color={getPriorityColor(task.priority) as any}
                                size="small"
                              />
                              {task.assignee && (
                                <Avatar sx={{ width: 24, height: 24 }}>
                                  {task.assignee.display_name[0]}
                                </Avatar>
                              )}
                            </Stack>
                            <Typography variant="subtitle2" sx={{ mb: 1 }}>
                              {task.title}
                            </Typography>
                            <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                              {task.description.length > 50 
                                ? `${task.description.substring(0, 50)}...`
                                : task.description
                              }
                            </Typography>
                            {task.due_date && (
                              <Chip 
                                icon={<ScheduleIcon />}
                                label={new Date(task.due_date).toLocaleDateString()}
                                size="small"
                                variant="outlined"
                              />
                            )}
                            {task.subtasks.length > 0 && (
                              <Box sx={{ mt: 1 }}>
                                <Typography variant="caption" color="text.secondary">
                                  Subtasks: {task.subtasks.filter(st => st.completed).length}/{task.subtasks.length}
                                </Typography>
                                <LinearProgress 
                                  variant="determinate" 
                                  value={(task.subtasks.filter(st => st.completed).length / task.subtasks.length) * 100}
                                  sx={{ mt: 0.5, height: 4 }}
                                />
                              </Box>
                            )}
                          </CardContent>
                        </Card>
                      </motion.div>
                    ))}
                </Stack>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>
    </Box>
  );

  const renderFilesTab = () => (
    <Box>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mb: 3 }}>
        <Typography variant="h5">Project Files</Typography>
        <Stack direction="row" spacing={1}>
          <Button
            variant="outlined"
            startIcon={<CreateFolderIcon />}
            onClick={() => onAction('create_folder')}
          >
            New Folder
          </Button>
          <Button
            variant="contained"
            startIcon={<CloudUploadIcon />}
            onClick={() => setFileDialogOpen(true)}
          >
            Upload Files
          </Button>
        </Stack>
      </Stack>

      <Grid container spacing={2}>
        {files.map((file) => (
          <Grid item xs={12} sm={6} md={4} lg={3} key={file.id}>
            <motion.div
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
            >
              <Card variant="outlined">
                <CardContent>
                  <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 1 }}>
                    <FileIcon color="primary" />
                    <Stack direction="row" spacing={0.5}>
                      {file.starred && <StarIcon color="warning" fontSize="small" />}
                      {file.shared && <ShareIcon color="info" fontSize="small" />}
                    </Stack>
                  </Stack>
                  <Typography variant="subtitle2" noWrap sx={{ mb: 1 }}>
                    {file.name}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                    {file.size}
                  </Typography>
                  <Stack direction="row" alignItems="center" spacing={1} sx={{ mb: 1 }}>
                    <Avatar sx={{ width: 20, height: 20 }}>
                      {file.uploaded_by.display_name[0]}
                    </Avatar>
                    <Typography variant="caption" color="text.secondary">
                      {file.uploaded_by.display_name}
                    </Typography>
                  </Stack>
                  <Typography variant="caption" color="text.secondary">
                    {new Date(file.uploaded_at).toLocaleDateString()}
                  </Typography>
                </CardContent>
                <CardActions>
                  <IconButton size="small" onClick={() => onAction('view_file', file)}>
                    <ViewIcon />
                  </IconButton>
                  <IconButton size="small" onClick={() => onAction('download_file', file)}>
                    <DownloadIcon />
                  </IconButton>
                  <IconButton size="small" onClick={() => onAction('share_file', file)}>
                    <ShareIcon />
                  </IconButton>
                </CardActions>
              </Card>
            </motion.div>
          </Grid>
        ))}
      </Grid>
    </Box>
  );

  const renderTimelineTab = () => (
    <Box>
      <Typography variant="h5" gutterBottom>Project Timeline</Typography>
      
      <Grid container spacing={3}>
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>Milestones & Deadlines</Typography>
              <Timeline>
                {milestones.map((milestone, index) => (
                  <TimelineItem key={milestone.id}>
                    <TimelineOppositeContent sx={{ m: 'auto 0' }} variant="body2" color="text.secondary">
                      {new Date(milestone.due_date).toLocaleDateString()}
                    </TimelineOppositeContent>
                    <TimelineSeparator>
                      <TimelineDot color={milestone.completed ? 'success' : 'primary'}>
                        <FlagIcon />
                      </TimelineDot>
                      {index < milestones.length - 1 && <TimelineConnector />}
                    </TimelineSeparator>
                    <TimelineContent sx={{ py: '12px', px: 2 }}>
                      <Typography variant="subtitle1" component="span">
                        {milestone.title}
                      </Typography>
                      <Typography>{milestone.description}</Typography>
                      <LinearProgress 
                        variant="determinate" 
                        value={milestone.completion_percentage}
                        sx={{ mt: 1 }}
                      />
                      <Typography variant="caption" color="text.secondary">
                        {milestone.completion_percentage}% complete
                      </Typography>
                    </TimelineContent>
                  </TimelineItem>
                ))}
              </Timeline>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>Task Timeline</Typography>
              <Timeline>
                {tasks
                  .filter(task => task.due_date)
                  .sort((a, b) => new Date(a.due_date!).getTime() - new Date(b.due_date!).getTime())
                  .map((task, index) => (
                    <TimelineItem key={task.id}>
                      <TimelineOppositeContent sx={{ m: 'auto 0' }} variant="body2" color="text.secondary">
                        {new Date(task.due_date!).toLocaleDateString()}
                      </TimelineOppositeContent>
                      <TimelineSeparator>
                        <TimelineDot color={getTaskStatusColor(task.status) as any}>
                          {task.status === 'completed' ? <CompleteIcon /> : <TaskIcon />}
                        </TimelineDot>
                        {index < tasks.filter(t => t.due_date).length - 1 && <TimelineConnector />}
                      </TimelineSeparator>
                      <TimelineContent sx={{ py: '12px', px: 2 }}>
                        <Typography variant="subtitle2" component="span">
                          {task.title}
                        </Typography>
                        <Typography variant="body2">{task.description}</Typography>
                        <Stack direction="row" spacing={1} sx={{ mt: 1 }}>
                          <Chip 
                            label={task.status.replace('_', ' ')} 
                            color={getTaskStatusColor(task.status) as any}
                            size="small"
                          />
                          <Chip 
                            label={task.priority} 
                            color={getPriorityColor(task.priority) as any}
                            size="small"
                            variant="outlined"
                          />
                        </Stack>
                      </TimelineContent>
                    </TimelineItem>
                  ))}
              </Timeline>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );

  const renderSettingsTab = () => (
    <Grid container spacing={3}>
      <Grid item xs={12} md={6}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>Project Settings</Typography>
            <Stack spacing={2}>
              <TextField
                label="Project Name"
                defaultValue={project.name}
                fullWidth
              />
              <TextField
                label="Description"
                defaultValue={project.description}
                multiline
                rows={3}
                fullWidth
              />
              <FormControl fullWidth>
                <InputLabel>Status</InputLabel>
                <Select value={project.status} label="Status">
                  <MenuItem value="active">Active</MenuItem>
                  <MenuItem value="on_hold">On Hold</MenuItem>
                  <MenuItem value="completed">Completed</MenuItem>
                  <MenuItem value="archived">Archived</MenuItem>
                </Select>
              </FormControl>
              <FormControl fullWidth>
                <InputLabel>Priority</InputLabel>
                <Select value={project.priority} label="Priority">
                  <MenuItem value="low">Low</MenuItem>
                  <MenuItem value="medium">Medium</MenuItem>
                  <MenuItem value="high">High</MenuItem>
                  <MenuItem value="critical">Critical</MenuItem>
                </Select>
              </FormControl>
            </Stack>
          </CardContent>
          <CardActions>
            <Button variant="contained">Save Changes</Button>
            <Button color="error">Delete Project</Button>
          </CardActions>
        </Card>
      </Grid>

      <Grid item xs={12} md={6}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>Project Features</Typography>
            <List>
              <ListItem>
                <ListItemIcon>
                  <SecurityIcon color={project.settings.version_control_enabled ? 'success' : 'disabled'} />
                </ListItemIcon>
                <ListItemText 
                  primary="Version Control"
                  secondary={project.settings.version_control_enabled ? 'Enabled' : 'Disabled'}
                />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <BackupIcon color={project.settings.backup_enabled ? 'success' : 'disabled'} />
                </ListItemIcon>
                <ListItemText 
                  primary="Automated Backup"
                  secondary={project.settings.backup_enabled ? 'Enabled' : 'Disabled'}
                />
              </ListItem>
              <ListItem>
                <ListItemIcon>
                  <PublicIcon />
                </ListItemIcon>
                <ListItemText 
                  primary="Visibility"
                  secondary="Private to project members"
                />
              </ListItem>
            </List>
          </CardContent>
        </Card>

        <Card sx={{ mt: 2 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>Storage Usage</Typography>
            <Box sx={{ mb: 2 }}>
              <Stack direction="row" justifyContent="space-between" sx={{ mb: 1 }}>
                <Typography variant="body2">
                  {project.storage_quota.used_gb.toFixed(1)} GB used
                </Typography>
                <Typography variant="body2">
                  {project.storage_quota.allocated_gb} GB total
                </Typography>
              </Stack>
              <LinearProgress 
                variant="determinate" 
                value={(project.storage_quota.used_gb / project.storage_quota.allocated_gb) * 100}
                sx={{ height: 8, borderRadius: 4 }}
              />
            </Box>
            <Button variant="outlined" size="small">
              Request More Storage
            </Button>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );

  return (
    <Box sx={{ height: '100%', overflow: 'auto' }}>
      {/* Project Header */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
        <Tabs value={activeTab} onChange={(_, newValue) => setActiveTab(newValue)}>
          <Tab label="Overview" />
          <Tab label="Tasks" />
          <Tab label="Files" />
          <Tab label="Timeline" />
          <Tab label="Settings" />
        </Tabs>
      </Box>

      {/* Tab Content */}
      <TabPanel value={activeTab} index={0}>
        {renderOverviewTab()}
      </TabPanel>
      <TabPanel value={activeTab} index={1}>
        {renderTasksTab()}
      </TabPanel>
      <TabPanel value={activeTab} index={2}>
        {renderFilesTab()}
      </TabPanel>
      <TabPanel value={activeTab} index={3}>
        {renderTimelineTab()}
      </TabPanel>
      <TabPanel value={activeTab} index={4}>
        {renderSettingsTab()}
      </TabPanel>

      {/* Speed Dial for Quick Actions */}
      <SpeedDial
        ariaLabel="Project Actions"
        sx={{ position: 'fixed', bottom: 24, right: 24 }}
        icon={<SpeedDialIcon openIcon={<CloseIcon />} />}
        open={speedDialOpen}
        onClose={() => setSpeedDialOpen(false)}
        onOpen={() => setSpeedDialOpen(true)}
      >
        <SpeedDialAction
          icon={<TaskIcon />}
          tooltipTitle="New Task"
          onClick={() => setTaskDialogOpen(true)}
        />
        <SpeedDialAction
          icon={<CloudUploadIcon />}
          tooltipTitle="Upload Files"
          onClick={() => setFileDialogOpen(true)}
        />
        <SpeedDialAction
          icon={<PersonAddIcon />}
          tooltipTitle="Add Member"
          onClick={() => onAction('add_member')}
        />
        <SpeedDialAction
          icon={<VideocamIcon />}
          tooltipTitle="Start Meeting"
          onClick={() => onCall('video')}
        />
      </SpeedDial>

      {/* Context Menu */}
      <Menu
        anchorEl={menuAnchor}
        open={Boolean(menuAnchor)}
        onClose={handleMenuClose}
      >
        <MenuItem onClick={() => { handleMenuClose(); onAction('edit_project'); }}>
          <ListItemIcon><EditIcon /></ListItemIcon>
          Edit Project
        </MenuItem>
        <MenuItem onClick={() => { handleMenuClose(); onAction('share_project'); }}>
          <ListItemIcon><ShareIcon /></ListItemIcon>
          Share Project
        </MenuItem>
        <MenuItem onClick={() => { handleMenuClose(); onAction('duplicate_project'); }}>
          <ListItemIcon><FileIcon /></ListItemIcon>
          Duplicate Project
        </MenuItem>
        <Divider />
        <MenuItem onClick={() => { handleMenuClose(); onAction('archive_project'); }}>
          <ListItemIcon><FolderIcon /></ListItemIcon>
          Archive Project
        </MenuItem>
        <MenuItem 
          onClick={() => { handleMenuClose(); onAction('delete_project'); }}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon><DeleteIcon color="error" /></ListItemIcon>
          Delete Project
        </MenuItem>
      </Menu>

      {/* Task Creation Dialog */}
      <Dialog open={taskDialogOpen} onClose={() => setTaskDialogOpen(false)} maxWidth="md" fullWidth>
        <DialogTitle>Create New Task</DialogTitle>
        <DialogContent>
          <Grid container spacing={2} sx={{ mt: 1 }}>
            <Grid item xs={12}>
              <TextField label="Task Title" fullWidth />
            </Grid>
            <Grid item xs={12}>
              <TextField label="Description" multiline rows={3} fullWidth />
            </Grid>
            <Grid item xs={6}>
              <FormControl fullWidth>
                <InputLabel>Priority</InputLabel>
                <Select defaultValue="medium">
                  <MenuItem value="low">Low</MenuItem>
                  <MenuItem value="medium">Medium</MenuItem>
                  <MenuItem value="high">High</MenuItem>
                  <MenuItem value="critical">Critical</MenuItem>
                </Select>
              </FormControl>
            </Grid>
            <Grid item xs={6}>
              <TextField type="date" label="Due Date" InputLabelProps={{ shrink: true }} fullWidth />
            </Grid>
            <Grid item xs={12}>
              <FormControl fullWidth>
                <InputLabel>Assignee</InputLabel>
                <Select>
                  {project.members.map((member) => (
                    <MenuItem key={member.user_id} value={member.user_id}>
                      {member.display_name}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setTaskDialogOpen(false)}>Cancel</Button>
          <Button variant="contained">Create Task</Button>
        </DialogActions>
      </Dialog>

      {/* File Upload Dialog */}
      <Dialog open={fileDialogOpen} onClose={() => setFileDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Upload Files</DialogTitle>
        <DialogContent>
          <Box 
            sx={{ 
              border: '2px dashed', 
              borderColor: 'divider', 
              borderRadius: 2, 
              p: 4, 
              textAlign: 'center',
              mt: 2
            }}
          >
            <CloudUploadIcon sx={{ fontSize: 48, color: 'action.disabled', mb: 2 }} />
            <Typography variant="h6" gutterBottom>Drag & Drop Files</Typography>
            <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
              or click to browse
            </Typography>
            <Button variant="outlined">Choose Files</Button>
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setFileDialogOpen(false)}>Cancel</Button>
          <Button variant="contained">Upload</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default ProjectView;