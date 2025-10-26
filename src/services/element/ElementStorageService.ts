import { EventEmitter } from 'events';
import { Element } from '../../types/element';

export interface FileUploadProgress {
  fileId: string;
  fileName: string;
  progress: number;
  total: number;
  speed: number;
}

export class ElementStorageService extends EventEmitter {
  private element: Element;
  private uploads = new Map<string, FileUploadProgress>();

  constructor(element: Element) {
    super();
    this.element = element;
  }

  async initialize(): Promise<void> {
    // Initialize storage service
  }

  async uploadFile(file: File): Promise<string> {
    if (!this.element.capabilities.storage) {
      throw new Error('Storage is not enabled for this element');
    }

    const fileId = `file_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    // Start upload tracking
    const progress: FileUploadProgress = {
      fileId,
      fileName: file.name,
      progress: 0,
      total: file.size,
      speed: 0,
    };

    this.uploads.set(fileId, progress);
    this.emit('upload-started', progress);

    try {
      // Simulate file upload progress
      const chunkSize = 1024 * 100; // 100KB chunks
      let uploaded = 0;

      while (uploaded < file.size) {
        await new Promise(resolve => setTimeout(resolve, 100)); // Simulate network delay

        uploaded = Math.min(uploaded + chunkSize, file.size);
        const updatedProgress = {
          ...progress,
          progress: uploaded,
        };
        this.uploads.set(fileId, updatedProgress);
        this.emit('upload-progress', updatedProgress);
      }

      // Store file data in localStorage (in real app, this would be uploaded to backend)
      const fileData = await file.arrayBuffer();
      localStorage.setItem(`element_${this.element.identity.id}_file_${fileId}`, JSON.stringify({
        data: Array.from(new Uint8Array(fileData)),
        name: file.name,
        type: file.type,
        size: file.size
      }));

      // Store file metadata
      const files = JSON.parse(localStorage.getItem(`element_${this.element.identity.id}_files`) || '[]');
      files.push({
        id: fileId,
        name: file.name,
        size: file.size,
        type: file.type,
        modified: new Date().toISOString(),
        path: `/${file.name}`
      });
      localStorage.setItem(`element_${this.element.identity.id}_files`, JSON.stringify(files));

      // Mark as complete
      const completeProgress = {
        ...progress,
        progress: file.size,
      };
      this.uploads.set(fileId, completeProgress);
      this.emit('upload-complete', completeProgress);

      // Clean up after a delay
      setTimeout(() => {
        this.uploads.delete(fileId);
      }, 5000);

      return fileId;
    } catch (error) {
      this.emit('upload-error', { fileId, error });
      this.uploads.delete(fileId);
      throw error;
    }
  }

  async downloadFile(fileId: string): Promise<Uint8Array> {
    if (!this.element.capabilities.storage) {
      throw new Error('Storage is not enabled for this element');
    }

    // Retrieve file data from localStorage
    const fileDataStr = localStorage.getItem(`element_${this.element.identity.id}_file_${fileId}`);
    if (!fileDataStr) {
      throw new Error('File not found');
    }

    const fileData = JSON.parse(fileDataStr);
    return new Uint8Array(fileData.data);
  }

  async listFiles(path: string = '/'): Promise<any[]> {
    if (!this.element.capabilities.storage) {
      throw new Error('Storage is not enabled for this element');
    }

    // Get stored files
    const files = JSON.parse(localStorage.getItem(`element_${this.element.identity.id}_files`) || '[]');

    // Get stored directories
    const directories = JSON.parse(localStorage.getItem(`element_${this.element.identity.id}_directories`) || '[]');

    // Filter files by path (basic implementation)
    const filteredFiles = files.filter((file: any) => {
      if (path === '/') {
        // For root directory, include files with exactly one path segment (e.g., '/filename')
        const pathSegments = file.path.split('/').filter((segment: string) => segment.length > 0);
        return pathSegments.length === 1;
      }
      return file.path.startsWith(path);
    });

    // Filter directories by path
    const filteredDirectories = directories.filter((dir: string) => {
      if (path === '/') return dir.split('/').length === 2; // Top-level directories
      return dir.startsWith(path) && dir !== path;
    }).map((dir: string) => ({
      id: `dir_${dir}`,
      name: dir.split('/').pop(),
      path: dir,
      type: 'directory',
      modified: new Date(),
      size: 0
    }));

    return [...filteredDirectories, ...filteredFiles];
  }

  async createDirectory(path: string): Promise<void> {
    if (!this.element.capabilities.storage) {
      throw new Error('Storage is not enabled for this element');
    }

    // Validate path
    if (!path.startsWith('/')) {
      throw new Error('Path must start with /');
    }

    if (path.includes('..') || path.includes('//')) {
      throw new Error('Invalid path format');
    }

    // For now, store directory metadata in local storage
    // In a real implementation, this would call the backend API
    const directories = JSON.parse(localStorage.getItem(`element_${this.element.identity.id}_directories`) || '[]');

    if (directories.includes(path)) {
      throw new Error('Directory already exists');
    }

    directories.push(path);
    localStorage.setItem(`element_${this.element.identity.id}_directories`, JSON.stringify(directories));

    console.log('Created directory:', path);
    this.emit('directory-created', { path });
  }

  async deleteFile(fileId: string): Promise<void> {
    if (!this.element.capabilities.storage) {
      throw new Error('Storage is not enabled for this element');
    }

    // Check if file exists in our mock storage
    const files = JSON.parse(localStorage.getItem(`element_${this.element.identity.id}_files`) || '[]');
    const fileIndex = files.findIndex((f: any) => f.id === fileId);

    if (fileIndex === -1) {
      throw new Error('File not found');
    }

    const file = files[fileIndex];

    // Remove file data from localStorage
    localStorage.removeItem(`element_${this.element.identity.id}_file_${fileId}`);

    // Remove file metadata
    files.splice(fileIndex, 1);
    localStorage.setItem(`element_${this.element.identity.id}_files`, JSON.stringify(files));

    console.log('Deleted file:', fileId);
    this.emit('file-deleted', { fileId, fileName: file.name });
  }

  getUploadProgress(fileId: string): FileUploadProgress | null {
    return this.uploads.get(fileId) || null;
  }

  getAllUploads(): FileUploadProgress[] {
    return Array.from(this.uploads.values());
  }

  async getStorageUsage(): Promise<{ used: number; total: number; available: number }> {
    // This would integrate with the element's storage limits
    const used = this.element.storage.usedSize;
    const total = this.element.storage.totalSize;
    const available = total - used;

    return { used, total, available };
  }

  async cleanup(): Promise<void> {
    this.uploads.clear();
    this.removeAllListeners();
  }
}