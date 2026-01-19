/**
 * Page object for ThreadListSidebar component.
 *
 * Provides methods to interact with the thread list sidebar,
 * including filtering, thread selection, and loading states.
 */
class ThreadListPage {
  /**
   * Thread list container element.
   */
  get container() {
    return $('.thread-list-sidebar');
  }

  /**
   * All thread list items.
   */
  get threads() {
    return $$('.thread-list-item');
  }

  /**
   * Skeleton loading placeholder.
   */
  get skeleton() {
    return $('.thread-list-skeleton');
  }

  /**
   * Empty state indicator.
   */
  get emptyState() {
    return $('.thread-list-empty');
  }

  /**
   * Filter tab: All.
   */
  get filterAll() {
    return $('[data-testid="filter-all"]');
  }

  /**
   * Filter tab: Entities (channels, groups, projects).
   */
  get filterEntities() {
    return $('[data-testid="filter-entities"]');
  }

  /**
   * Filter tab: Contacts (DMs).
   */
  get filterContacts() {
    return $('[data-testid="filter-contacts"]');
  }

  /**
   * Filter tab: Unread.
   */
  get filterUnread() {
    return $('[data-testid="filter-unread"]');
  }

  /**
   * Wait for thread list to finish loading (skeleton disappears).
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
   * Select a thread by index.
   * @param {number} index - Thread index (0-based)
   */
  async selectThread(index) {
    const threads = await this.threads;
    if (threads.length > index) {
      await threads[index].click();
    }
  }

  /**
   * Get the number of threads displayed.
   * @returns {Promise<number>}
   */
  async getThreadCount() {
    const threads = await this.threads;
    return threads.length;
  }

  /**
   * Get thread display names.
   * @returns {Promise<string[]>}
   */
  async getThreadNames() {
    const threads = await this.threads;
    const names = [];
    for (const thread of threads) {
      const nameEl = await thread.$('.thread-name');
      if (await nameEl.isExisting()) {
        names.push(await nameEl.getText());
      }
    }
    return names;
  }

  /**
   * Check if thread list shows empty state.
   * @returns {Promise<boolean>}
   */
  async isEmpty() {
    const emptyState = await this.emptyState;
    return emptyState.isDisplayed();
  }

  /**
   * Filter threads by type.
   * @param {'all'|'entities'|'contacts'|'unread'} filter
   */
  async filterBy(filter) {
    switch (filter) {
      case 'all':
        await (await this.filterAll).click();
        break;
      case 'entities':
        await (await this.filterEntities).click();
        break;
      case 'contacts':
        await (await this.filterContacts).click();
        break;
      case 'unread':
        await (await this.filterUnread).click();
        break;
    }
    // Wait for filter to apply
    await browser.pause(500);
  }
}

export default new ThreadListPage();
