/**
 * Page object for Kanban Board component.
 *
 * Provides methods to interact with the kanban board,
 * including columns, cards, filters, and drag-and-drop.
 */
class KanbanPage {
  /**
   * Main board view container.
   */
  get container() {
    return $('.board-view');
  }

  /**
   * Board header with title.
   */
  get boardHeader() {
    return $('h1, .board-header');
  }

  /**
   * Board description (if visible).
   */
  get boardDescription() {
    return $('p.text-slate-400');
  }

  /**
   * Filter toggle button.
   */
  get filterToggleBtn() {
    return $('button*=Filter, button[aria-label*="filter"]');
  }

  /**
   * Filter panel (when visible).
   */
  get filterPanel() {
    return $('.filter-panel, [aria-label="Filter options"]');
  }

  /**
   * Analytics button.
   */
  get analyticsBtn() {
    return $('button*=Analytics, button[aria-label*="analytics"]');
  }

  /**
   * Analytics dashboard modal.
   */
  get analyticsDashboard() {
    return $('.analytics-dashboard');
  }

  /**
   * All columns on the board.
   */
  get columns() {
    return $$('.kanban-column, [role="list"]');
  }

  /**
   * Column headers.
   */
  get columnHeaders() {
    return $$('.kanban-column-header, h2');
  }

  /**
   * All cards on the board.
   */
  get cards() {
    return $$('.kanban-card, [role="listitem"]');
  }

  /**
   * Add column button.
   */
  get addColumnBtn() {
    return $('button*=Add Column, button*=column, button[aria-label*="add column"]');
  }

  /**
   * Add card button (in any column).
   */
  get addCardBtns() {
    return $$('button*=Add Card, button[aria-label*="add card"]');
  }

  /**
   * Card detail modal.
   */
  get cardDetailModal() {
    return $('.card-detail-modal, [role="dialog"]');
  }

  /**
   * Card detail modal close button.
   */
  get modalCloseBtn() {
    return $('[aria-label="Close"], button*=Close');
  }

  /**
   * Conflict banner (CRDT conflicts).
   */
  get conflictBanner() {
    return $('.conflict-banner');
  }

  /**
   * Loading skeleton.
   */
  get skeleton() {
    return $('.board-view-skeleton, [aria-busy="true"]');
  }

  /**
   * Error display.
   */
  get errorDisplay() {
    return $('*=Failed to load');
  }

  /**
   * Retry button.
   */
  get retryBtn() {
    return $('button=Retry');
  }

  /**
   * Swimlane container (if swimlane mode active).
   */
  get swimlanes() {
    return $$('.swimlane');
  }

  /**
   * Aria-live region for announcements.
   */
  get announcements() {
    return $('[role="status"][aria-live="polite"]');
  }

  /**
   * Wait for board to load.
   * @param {number} timeout - Maximum wait time in ms
   */
  async waitForLoad(timeout = 10000) {
    const container = await this.container;
    await container.waitForExist({ timeout });

    // Wait for skeleton to disappear
    const skeleton = await this.skeleton;
    if (await skeleton.isExisting()) {
      await skeleton.waitForDisplayed({ reverse: true, timeout });
    }
  }

  /**
   * Navigate to kanban board route.
   * @param {string} boardId - Board ID (default 'test-board')
   */
  async navigate(boardId = 'test-board') {
    await browser.url(`tauri://localhost/kanban/${boardId}`);
    await this.waitForLoad();
  }

  /**
   * Get the board name.
   * @returns {Promise<string>}
   */
  async getBoardName() {
    const header = await this.boardHeader;
    if (await header.isExisting()) {
      return header.getText();
    }
    return '';
  }

  /**
   * Get number of columns.
   * @returns {Promise<number>}
   */
  async getColumnCount() {
    const columns = await this.columns;
    return columns.length;
  }

  /**
   * Get column names.
   * @returns {Promise<string[]>}
   */
  async getColumnNames() {
    const headers = await this.columnHeaders;
    const names = [];
    for (const header of headers) {
      const text = await header.getText();
      if (text) {
        names.push(text);
      }
    }
    return names;
  }

  /**
   * Get number of cards on the board.
   * @returns {Promise<number>}
   */
  async getCardCount() {
    const cards = await this.cards;
    return cards.length;
  }

  /**
   * Get cards in a specific column by index.
   * @param {number} columnIndex - Column index (0-based)
   * @returns {Promise<WebdriverIO.Element[]>}
   */
  async getCardsInColumn(columnIndex) {
    const columns = await this.columns;
    if (columns.length > columnIndex) {
      return columns[columnIndex].$$('.kanban-card, [role="listitem"]');
    }
    return [];
  }

  /**
   * Click a card to open detail modal.
   * @param {number} cardIndex - Card index (0-based)
   */
  async openCard(cardIndex) {
    const cards = await this.cards;
    if (cards.length > cardIndex) {
      await cards[cardIndex].click();
      await browser.pause(300);
    }
  }

  /**
   * Close the card detail modal.
   */
  async closeModal() {
    const closeBtn = await this.modalCloseBtn;
    if (await closeBtn.isExisting()) {
      await closeBtn.click();
      await browser.pause(300);
    }
  }

  /**
   * Check if card detail modal is open.
   * @returns {Promise<boolean>}
   */
  async isModalOpen() {
    const modal = await this.cardDetailModal;
    return modal.isDisplayed();
  }

  /**
   * Toggle filter panel.
   */
  async toggleFilters() {
    const btn = await this.filterToggleBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * Check if filter panel is visible.
   * @returns {Promise<boolean>}
   */
  async isFilterPanelVisible() {
    const panel = await this.filterPanel;
    if (await panel.isExisting()) {
      return panel.isDisplayed();
    }
    return false;
  }

  /**
   * Open analytics dashboard.
   */
  async openAnalytics() {
    const btn = await this.analyticsBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * Check if analytics dashboard is visible.
   * @returns {Promise<boolean>}
   */
  async isAnalyticsVisible() {
    const dashboard = await this.analyticsDashboard;
    if (await dashboard.isExisting()) {
      return dashboard.isDisplayed();
    }
    return false;
  }

  /**
   * Add a new column.
   */
  async clickAddColumn() {
    const btn = await this.addColumnBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * Add a new card to a column.
   * @param {number} columnIndex - Column index to add card to
   */
  async clickAddCard(columnIndex = 0) {
    const btns = await this.addCardBtns;
    if (btns.length > columnIndex) {
      await btns[columnIndex].click();
      await browser.pause(300);
    }
  }

  /**
   * Start dragging a card (for drag preview test).
   * @param {number} cardIndex - Card index
   */
  async startDrag(cardIndex) {
    const cards = await this.cards;
    if (cards.length > cardIndex) {
      // Use JavaScript to fire dragstart event
      await browser.execute((el) => {
        const event = new DragEvent('dragstart', {
          bubbles: true,
          cancelable: true,
        });
        el.dispatchEvent(event);
      }, cards[cardIndex]);
      await browser.pause(100);
    }
  }

  /**
   * Check if board has errors.
   * @returns {Promise<boolean>}
   */
  async hasError() {
    const error = await this.errorDisplay;
    return error.isExisting();
  }

  /**
   * Retry loading.
   */
  async retry() {
    const btn = await this.retryBtn;
    if (await btn.isExisting()) {
      await btn.click();
      await browser.pause(500);
    }
  }

  /**
   * Check for CRDT conflict banners.
   * @returns {Promise<boolean>}
   */
  async hasConflicts() {
    const banner = await this.conflictBanner;
    return banner.isExisting();
  }

  /**
   * Check if swimlane mode is active.
   * @returns {Promise<boolean>}
   */
  async isSwimlaneMode() {
    const swimlanes = await this.swimlanes;
    return swimlanes.length > 0;
  }
}

export default new KanbanPage();
