/**
 * Page object for Drive Browser component.
 *
 * Provides methods to interact with the drive browser,
 * including file listing, folder navigation, uploads, and previews.
 */
class DrivePage {
  /**
   * Main drive browser container.
   */
  get container() {
    return $('.drive-browser');
  }

  /**
   * Disk type tabs (Private, Public, Shared).
   */
  get diskTabs() {
    return $$('[role="tab"]');
  }

  /**
   * Private disk tab.
   */
  get tabPrivate() {
    return $('[role="tab"]:nth-child(1)');
  }

  /**
   * Public disk tab.
   */
  get tabPublic() {
    return $('[role="tab"]:nth-child(2)');
  }

  /**
   * Shared disk tab.
   */
  get tabShared() {
    return $('[role="tab"]:nth-child(3)');
  }

  /**
   * Upload Files button.
   */
  get uploadBtn() {
    return $('button=Upload Files');
  }

  /**
   * New Folder button.
   */
  get newFolderBtn() {
    return $('button=New Folder');
  }

  /**
   * Uploads panel button.
   */
  get uploadsBtn() {
    return $('button=Uploads');
  }

  /**
   * List view mode button.
   */
  get listViewBtn() {
    return $('button*=☰');
  }

  /**
   * Grid view mode button.
   */
  get gridViewBtn() {
    return $('button*=⊞');
  }

  /**
   * Tree toggle button.
   */
  get treeToggleBtn() {
    return $('button*=🌲');
  }

  /**
   * Breadcrumb navigation.
   */
  get breadcrumb() {
    return $('nav[aria-label="Breadcrumb"]');
  }

  /**
   * Breadcrumb buttons (including root).
   */
  get breadcrumbButtons() {
    return $$('nav[aria-label="Breadcrumb"] button');
  }

  /**
   * File list items (table rows or grid items).
   */
  get fileItems() {
    return $$('[role="row"], .file-grid-item');
  }

  /**
   * Tree view sidebar.
   */
  get treeView() {
    return $('.drive-tree, .w-64');
  }

  /**
   * Preview panel.
   */
  get previewPanel() {
    return $('.drive-preview, .w-80');
  }

  /**
   * Loading skeleton.
   */
  get skeleton() {
    return $('[aria-label="Loading files"]');
  }

  /**
   * Error alert panel.
   */
  get errorPanel() {
    return $('[role="alert"]');
  }

  /**
   * Retry button in error panel.
   */
  get retryBtn() {
    return $('button=Retry');
  }

  /**
   * Empty state when folder is empty.
   */
  get emptyState() {
    return $('*=This folder is empty');
  }

  /**
   * Quota meter.
   */
  get quotaMeter() {
    return $('.quota-meter, .border-t .h-2');
  }

  /**
   * Drag overlay when dragging files.
   */
  get dragOverlay() {
    return $('*=Drop files to upload');
  }

  /**
   * Upload progress panel.
   */
  get uploadProgressPanel() {
    return $('.upload-progress-panel, [aria-label*="upload"]');
  }

  /**
   * Wait for drive to finish loading.
   * @param {number} timeout - Maximum wait time in ms
   */
  async waitForLoad(timeout = 10000) {
    const container = await this.container;
    await container.waitForExist({ timeout });

    // Wait for skeleton to disappear (if present)
    const skeleton = await this.skeleton;
    if (await skeleton.isExisting()) {
      await skeleton.waitForDisplayed({ reverse: true, timeout });
    }
  }

  /**
   * Navigate to drive route.
   * @param {string} entityId - Entity owning the drive (default 'test')
   * @param {string} diskType - Disk type (private, public, shared)
   */
  async navigate(entityId = 'test', diskType = 'private') {
    await browser.url(`tauri://localhost/entity/${entityId}/drive?disk=${diskType}`);
    await this.waitForLoad();
  }

  /**
   * Switch to a different disk type.
   * @param {'private'|'public'|'shared'} disk - Disk type
   */
  async switchDisk(disk) {
    const tabs = await this.diskTabs;
    const diskIndex = { private: 0, public: 1, shared: 2 };
    if (tabs.length > diskIndex[disk]) {
      await tabs[diskIndex[disk]].click();
      await browser.pause(500);
    }
  }

  /**
   * Get the currently selected disk type.
   * @returns {Promise<string>}
   */
  async getSelectedDisk() {
    const tabs = await this.diskTabs;
    for (const tab of tabs) {
      const selected = await tab.getAttribute('aria-selected');
      if (selected === 'true') {
        return (await tab.getText()).toLowerCase();
      }
    }
    return '';
  }

  /**
   * Click the upload button.
   */
  async clickUpload() {
    const btn = await this.uploadBtn;
    await btn.click();
  }

  /**
   * Click the new folder button.
   */
  async clickNewFolder() {
    const btn = await this.newFolderBtn;
    await btn.click();
  }

  /**
   * Get count of visible file items.
   * @returns {Promise<number>}
   */
  async getFileCount() {
    const items = await this.fileItems;
    return items.length;
  }

  /**
   * Select a file by index.
   * @param {number} index - File index (0-based)
   */
  async selectFile(index) {
    const items = await this.fileItems;
    if (items.length > index) {
      await items[index].click();
    }
  }

  /**
   * Double-click to open a file/folder by index.
   * @param {number} index - File index (0-based)
   */
  async openFile(index) {
    const items = await this.fileItems;
    if (items.length > index) {
      await items[index].doubleClick();
    }
  }

  /**
   * Get file names visible in the list.
   * @returns {Promise<string[]>}
   */
  async getFileNames() {
    const items = await this.fileItems;
    const names = [];
    for (const item of items) {
      const text = await item.getText();
      if (text) {
        // First line is usually the name
        names.push(text.split('\n')[0]);
      }
    }
    return names;
  }

  /**
   * Navigate via breadcrumb.
   * @param {number} index - Breadcrumb index (0 = root)
   */
  async navigateBreadcrumb(index) {
    const buttons = await this.breadcrumbButtons;
    if (buttons.length > index) {
      await buttons[index].click();
      await browser.pause(500);
    }
  }

  /**
   * Get current path from breadcrumb.
   * @returns {Promise<string>}
   */
  async getCurrentPath() {
    const buttons = await this.breadcrumbButtons;
    const segments = [];
    for (const btn of buttons) {
      const text = await btn.getText();
      if (text && text !== '/') {
        segments.push(text);
      }
    }
    return '/' + segments.join('/');
  }

  /**
   * Toggle tree view visibility.
   */
  async toggleTreeView() {
    const btn = await this.treeToggleBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * Check if tree view is visible.
   * @returns {Promise<boolean>}
   */
  async isTreeViewVisible() {
    const tree = await this.treeView;
    return tree.isDisplayed();
  }

  /**
   * Check if preview panel is visible.
   * @returns {Promise<boolean>}
   */
  async isPreviewVisible() {
    const panel = await this.previewPanel;
    if (await panel.isExisting()) {
      return panel.isDisplayed();
    }
    return false;
  }

  /**
   * Switch view mode.
   * @param {'list'|'grid'} mode - View mode
   */
  async setViewMode(mode) {
    const btn = mode === 'list' ? await this.listViewBtn : await this.gridViewBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * Check if in empty state.
   * @returns {Promise<boolean>}
   */
  async isEmpty() {
    const emptyState = await this.emptyState;
    return emptyState.isExisting();
  }

  /**
   * Check if error state is displayed.
   * @returns {Promise<boolean>}
   */
  async hasError() {
    const errorPanel = await this.errorPanel;
    return errorPanel.isExisting();
  }

  /**
   * Retry after error.
   */
  async retry() {
    const btn = await this.retryBtn;
    if (await btn.isExisting()) {
      await btn.click();
      await browser.pause(500);
    }
  }

  /**
   * Open uploads panel.
   */
  async openUploadsPanel() {
    const btn = await this.uploadsBtn;
    await btn.click();
    await browser.pause(300);
  }
}

export default new DrivePage();
