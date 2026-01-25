/**
 * Page object for Canvas View component.
 *
 * Provides methods to interact with the collaborative canvas,
 * including drawing tools, elements, and real-time features.
 */
class CanvasPage {
  /**
   * Main canvas view container.
   */
  get container() {
    return $('.canvas-view');
  }

  /**
   * Canvas toolbar.
   */
  get toolbar() {
    return $('.canvas-toolbar, [role="toolbar"]');
  }

  /**
   * Drawing tools group.
   */
  get drawingTools() {
    return $('[role="group"][aria-label="Drawing tools"]');
  }

  /**
   * All tool buttons.
   */
  get toolButtons() {
    return $$('.canvas-toolbar button[aria-label]');
  }

  /**
   * Undo button.
   */
  get undoBtn() {
    return $('[aria-label="Undo"]');
  }

  /**
   * Redo button.
   */
  get redoBtn() {
    return $('[aria-label="Redo"]');
  }

  /**
   * SVG canvas area.
   */
  get svgCanvas() {
    return $('svg.canvas-content, svg');
  }

  /**
   * Canvas elements (rendered shapes).
   */
  get elements() {
    return $$('svg .canvas-element, svg rect, svg ellipse, svg path');
  }

  /**
   * Selected elements.
   */
  get selectedElements() {
    return $$('[data-selected="true"], .selected');
  }

  /**
   * Grid overlay (when visible).
   */
  get gridOverlay() {
    return $('defs pattern, .grid-overlay');
  }

  /**
   * Remote cursors (collaborative).
   */
  get remoteCursors() {
    return $('.remote-cursors');
  }

  /**
   * Individual remote cursor indicators.
   */
  get cursorIndicators() {
    return $$('.remote-cursor');
  }

  /**
   * History indicator/scrubber.
   */
  get historyIndicator() {
    return $('.history-indicator');
  }

  /**
   * Sync status badge.
   */
  get syncStatusBadge() {
    return $('.sync-status-badge, [aria-label*="sync"]');
  }

  /**
   * Layer panel.
   */
  get layerPanel() {
    return $('.layer-panel');
  }

  /**
   * Offline indicator.
   */
  get offlineIndicator() {
    return $('.offline-indicator');
  }

  /**
   * Color picker/palette.
   */
  get colorPicker() {
    return $('[aria-label*="color"], .color-picker');
  }

  /**
   * Keyboard shortcuts help overlay.
   */
  get helpOverlay() {
    return $('.keyboard-help, [aria-label*="keyboard shortcuts"]');
  }

  /**
   * Loading overlay.
   */
  get loadingOverlay() {
    return $('.loading-overlay, [aria-busy="true"]');
  }

  /**
   * Aria-live status region.
   */
  get statusRegion() {
    return $('[role="status"][aria-live="polite"]');
  }

  /**
   * Export button/menu.
   */
  get exportBtn() {
    return $('button*=Export, [aria-label*="export"]');
  }

  /**
   * Zoom controls.
   */
  get zoomControls() {
    return $('.zoom-controls, [aria-label*="zoom"]');
  }

  /**
   * Wait for canvas to load.
   * @param {number} timeout - Maximum wait time in ms
   */
  async waitForLoad(timeout = 10000) {
    const container = await this.container;
    await container.waitForExist({ timeout });

    // Wait for loading overlay to disappear
    const loading = await this.loadingOverlay;
    if (await loading.isExisting()) {
      await loading.waitForDisplayed({ reverse: true, timeout });
    }
  }

  /**
   * Navigate to canvas route.
   * @param {string} entityId - Entity ID (default 'test')
   */
  async navigate(entityId = 'test') {
    await browser.url(`tauri://localhost/entity/${entityId}/canvas`);
    await this.waitForLoad();
  }

  /**
   * Get the currently selected tool.
   * @returns {Promise<string>}
   */
  async getSelectedTool() {
    const buttons = await this.toolButtons;
    for (const btn of buttons) {
      const className = await btn.getAttribute('class');
      if (className.includes('emerald')) {
        return btn.getAttribute('aria-label');
      }
    }
    return '';
  }

  /**
   * Select a tool by label.
   * @param {string} label - Tool label (e.g., 'Select', 'Rectangle')
   */
  async selectTool(label) {
    const btn = await $(`[aria-label="${label}"]`);
    if (await btn.isExisting()) {
      await btn.click();
      await browser.pause(100);
    }
  }

  /**
   * Get number of tool buttons.
   * @returns {Promise<number>}
   */
  async getToolCount() {
    const buttons = await this.toolButtons;
    return buttons.length;
  }

  /**
   * Check if undo is available.
   * @returns {Promise<boolean>}
   */
  async canUndo() {
    const btn = await this.undoBtn;
    const disabled = await btn.getAttribute('disabled');
    return disabled === null;
  }

  /**
   * Check if redo is available.
   * @returns {Promise<boolean>}
   */
  async canRedo() {
    const btn = await this.redoBtn;
    const disabled = await btn.getAttribute('disabled');
    return disabled === null;
  }

  /**
   * Click undo button.
   */
  async undo() {
    const btn = await this.undoBtn;
    if (await this.canUndo()) {
      await btn.click();
      await browser.pause(100);
    }
  }

  /**
   * Click redo button.
   */
  async redo() {
    const btn = await this.redoBtn;
    if (await this.canRedo()) {
      await btn.click();
      await browser.pause(100);
    }
  }

  /**
   * Get number of elements on canvas.
   * @returns {Promise<number>}
   */
  async getElementCount() {
    const elements = await this.elements;
    return elements.length;
  }

  /**
   * Click on the canvas at a position.
   * @param {number} x - X coordinate
   * @param {number} y - Y coordinate
   */
  async clickCanvas(x, y) {
    const svg = await this.svgCanvas;
    await svg.click({ x, y });
    await browser.pause(100);
  }

  /**
   * Check if canvas has focus.
   * @returns {Promise<boolean>}
   */
  async hasFocus() {
    const container = await this.container;
    const focused = await browser.execute(() => {
      return document.activeElement?.className || '';
    });
    return focused.includes('canvas-view');
  }

  /**
   * Focus the canvas.
   */
  async focus() {
    const container = await this.container;
    await container.click();
  }

  /**
   * Press keyboard shortcut.
   * @param {string} keys - Keys to press (e.g., 'Control+z')
   */
  async pressKeys(keys) {
    await browser.keys(keys.split('+'));
  }

  /**
   * Check if help overlay is visible.
   * @returns {Promise<boolean>}
   */
  async isHelpVisible() {
    const help = await this.helpOverlay;
    if (await help.isExisting()) {
      return help.isDisplayed();
    }
    return false;
  }

  /**
   * Toggle help overlay with ? key.
   */
  async toggleHelp() {
    await this.focus();
    await browser.keys(['?']);
    await browser.pause(300);
  }

  /**
   * Check if remote cursors are visible.
   * @returns {Promise<boolean>}
   */
  async hasRemoteCursors() {
    const cursors = await this.remoteCursors;
    return cursors.isExisting();
  }

  /**
   * Get number of remote cursor indicators.
   * @returns {Promise<number>}
   */
  async getRemoteCursorCount() {
    const cursors = await this.cursorIndicators;
    return cursors.length;
  }

  /**
   * Check if sync status badge shows synced.
   * @returns {Promise<boolean>}
   */
  async isSynced() {
    const badge = await this.syncStatusBadge;
    if (await badge.isExisting()) {
      const text = await badge.getText();
      const className = await badge.getAttribute('class');
      return text.includes('synced') || className.includes('emerald');
    }
    return true; // Default to synced if no badge
  }

  /**
   * Check if canvas is loading.
   * @returns {Promise<boolean>}
   */
  async isLoading() {
    const loading = await this.loadingOverlay;
    return loading.isDisplayed();
  }

  /**
   * Get the status announcement text.
   * @returns {Promise<string>}
   */
  async getStatusText() {
    const status = await this.statusRegion;
    if (await status.isExisting()) {
      return status.getText();
    }
    return '';
  }
}

export default new CanvasPage();
