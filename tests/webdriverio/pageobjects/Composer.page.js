/**
 * Page object for MessageComposer component.
 *
 * Provides methods to interact with the message composer,
 * including typing messages, sending, and reply flow.
 */
class ComposerPage {
  /**
   * Message input textarea.
   */
  get textarea() {
    return $('.composer-textarea');
  }

  /**
   * Send button.
   */
  get sendBtn() {
    return $('.composer-send-btn');
  }

  /**
   * Reply indicator showing which message is being replied to.
   */
  get replyIndicator() {
    return $('.reply-indicator');
  }

  /**
   * Cancel reply button.
   */
  get cancelReplyBtn() {
    return $('.reply-cancel');
  }

  /**
   * Sending indicator (shown while message is being sent).
   */
  get sendingIndicator() {
    return $('.composer-sending');
  }

  /**
   * Error message display.
   */
  get errorMessage() {
    return $('.composer-error');
  }

  /**
   * Type text into the composer.
   * @param {string} text - Text to type
   */
  async type(text) {
    const textarea = await this.textarea;
    await textarea.waitForExist({ timeout: 5000 });
    await textarea.setValue(text);
  }

  /**
   * Send a message (type + click send).
   * @param {string} text - Message text
   */
  async sendMessage(text) {
    await this.type(text);
    const sendBtn = await this.sendBtn;
    await sendBtn.waitForClickable({ timeout: 5000 });
    await sendBtn.click();
  }

  /**
   * Wait for message to be sent (textarea clears).
   * @param {number} timeout - Maximum wait time in ms
   */
  async waitForSent(timeout = 5000) {
    await browser.waitUntil(
      async () => {
        const textarea = await this.textarea;
        const value = await textarea.getValue();
        return value === '';
      },
      { timeout, timeoutMsg: 'Message was not sent in time' }
    );
  }

  /**
   * Check if reply indicator is displayed.
   * @returns {Promise<boolean>}
   */
  async isReplying() {
    const indicator = await this.replyIndicator;
    return indicator.isDisplayed();
  }

  /**
   * Cancel the current reply.
   */
  async cancelReply() {
    const cancelBtn = await this.cancelReplyBtn;
    await cancelBtn.click();
  }

  /**
   * Get current text in composer.
   * @returns {Promise<string>}
   */
  async getText() {
    const textarea = await this.textarea;
    return textarea.getValue();
  }

  /**
   * Check if composer is in sending state.
   * @returns {Promise<boolean>}
   */
  async isSending() {
    const indicator = await this.sendingIndicator;
    return indicator.isDisplayed();
  }

  /**
   * Check if there's an error displayed.
   * @returns {Promise<boolean>}
   */
  async hasError() {
    const error = await this.errorMessage;
    return error.isDisplayed();
  }

  /**
   * Get error message text.
   * @returns {Promise<string>}
   */
  async getErrorText() {
    const error = await this.errorMessage;
    return error.getText();
  }

  /**
   * Press Enter to send (Shift+Enter for newline is handled by component).
   */
  async pressEnterToSend() {
    const textarea = await this.textarea;
    await textarea.click();
    await browser.keys(['Enter']);
  }
}

export default new ComposerPage();
