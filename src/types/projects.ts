// Project and Issue types matching Rust implementation

export interface Project {
  id: string;
  org_id: string;
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  created_at: number;
  created_by: string;
}

export type IssueStatus = 'backlog' | 'todo' | 'in-progress' | 'done' | 'canceled';

export type IssuePriority = 'urgent' | 'high' | 'medium' | 'low';

export interface Issue {
  id: string;
  project_id: string;
  title: string;
  description?: string;
  status: IssueStatus;
  priority: IssuePriority;
  assignee_id?: string;
  reporter_id: string;
  created_at: number;
  updated_at?: number;
}

export interface IssueComment {
  id: string;
  issue_id: string;
  author_id: string;
  content: string;
  created_at: number;
  updated_at?: number;
}

export interface CreateProjectRequest {
  org_id: string;
  name: string;
  description?: string;
  icon?: string;
  color?: string;
  created_by: string;
}

export interface CreateIssueRequest {
  project_id: string;
  title: string;
  description?: string;
  priority: IssuePriority;
  reporter_id: string;
}

// Kanban board column definition
export interface KanbanColumn {
  id: IssueStatus;
  title: string;
  issues: Issue[];
}

// Issue status colors for UI
export const issueStatusColors: Record<IssueStatus, string> = {
  backlog: '#94a3b8', // gray
  todo: '#3b82f6', // blue
  'in-progress': '#f59e0b', // orange
  done: '#10b981', // green
  canceled: '#6b7280', // gray-dark
};

// Priority colors for UI
export const issuePriorityColors: Record<IssuePriority, string> = {
  urgent: '#dc2626', // red
  high: '#ea580c', // orange-red
  medium: '#f59e0b', // orange
  low: '#64748b', // gray
};
