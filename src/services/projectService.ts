import { invoke } from '@tauri-apps/api/core';
import type {
  Project,
  Issue,
  IssueComment,
  IssueStatus,
  IssuePriority,
  CreateProjectRequest,
  CreateIssueRequest,
} from '../types/projects';

/**
 * Project Service - Frontend interface to Tauri project/issue commands
 * All operations work offline via CRDT and sync automatically
 */
export class ProjectService {
  // === Project Operations ===

  async createProject(request: CreateProjectRequest): Promise<Project> {
    return invoke<Project>('create_project', { request });
  }

  async getProject(projectId: string): Promise<Project | null> {
    return invoke<Project | null>('get_project', { projectId });
  }

  async listProjects(orgId: string): Promise<Project[]> {
    return invoke<Project[]>('list_projects', { orgId });
  }

  // === Issue Operations ===

  async createIssue(request: CreateIssueRequest): Promise<Issue> {
    return invoke<Issue>('create_issue', { request });
  }

  async getIssue(issueId: string): Promise<Issue | null> {
    return invoke<Issue | null>('get_issue', { issueId });
  }

  async listIssues(projectId: string): Promise<Issue[]> {
    return invoke<Issue[]>('list_issues', { projectId });
  }

  async listIssuesByStatus(
    projectId: string,
    status: IssueStatus
  ): Promise<Issue[]> {
    return invoke<Issue[]>('list_issues_by_status', { projectId, status });
  }

  // === Issue Updates ===

  async updateStatus(issueId: string, newStatus: IssueStatus): Promise<void> {
    return invoke<void>('update_issue_status', { issueId, newStatus });
  }

  async assignIssue(issueId: string, assigneeId: string): Promise<void> {
    return invoke<void>('assign_issue', { issueId, assigneeId });
  }

  async updatePriority(
    issueId: string,
    priority: IssuePriority
  ): Promise<void> {
    return invoke<void>('update_issue_priority', { issueId, priority });
  }

  // === Comments ===

  async addComment(
    issueId: string,
    authorId: string,
    content: string
  ): Promise<IssueComment> {
    return invoke<IssueComment>('add_issue_comment', {
      issueId,
      authorId,
      content,
    });
  }

  async getComments(issueId: string): Promise<IssueComment[]> {
    return invoke<IssueComment[]>('get_issue_comments', { issueId });
  }

  // === Sync Operations ===

  async getSyncUpdate(issueId: string): Promise<Uint8Array> {
    return invoke<Uint8Array>('get_issue_sync_update', { issueId });
  }

  async applySyncUpdate(issueId: string, update: Uint8Array): Promise<void> {
    return invoke<void>('apply_issue_sync_update', { issueId, update });
  }

  // === Kanban Helper ===

  /**
   * Get all issues for a project organized by status (Kanban board)
   */
  async getKanbanBoard(projectId: string): Promise<{
    backlog: Issue[];
    todo: Issue[];
    'in-progress': Issue[];
    done: Issue[];
    canceled: Issue[];
  }> {
    const statuses: IssueStatus[] = [
      'backlog',
      'todo',
      'in-progress',
      'done',
      'canceled',
    ];

    const results = await Promise.all(
      statuses.map((status) => this.listIssuesByStatus(projectId, status))
    );

    return {
      backlog: results[0],
      todo: results[1],
      'in-progress': results[2],
      done: results[3],
      canceled: results[4],
    };
  }
}

// Singleton instance
export const projectService = new ProjectService();
