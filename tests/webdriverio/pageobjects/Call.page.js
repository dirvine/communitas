/**
 * Page object for Call View component.
 *
 * Provides methods to interact with the call interface,
 * including call controls, participant grid, and call status.
 */
class CallPage {
  /**
   * Main call view container.
   */
  get container() {
    return $('.call-view');
  }

  /**
   * Mini call view (picture-in-picture style).
   */
  get miniCallView() {
    return $('.mini-call-view');
  }

  /**
   * Call status bar (shown when navigating away from call).
   */
  get callStatusBar() {
    return $('.call-status-bar');
  }

  /**
   * Call controls toolbar.
   */
  get controlsToolbar() {
    return $('.call-controls, [role="toolbar"]');
  }

  /**
   * Mute microphone button.
   */
  get muteBtn() {
    return $('[aria-label*="mute"], [aria-label*="Mute"]');
  }

  /**
   * Video toggle button.
   */
  get videoBtn() {
    return $('[aria-label*="camera"], [aria-label*="Camera"]');
  }

  /**
   * Screen share button.
   */
  get screenShareBtn() {
    return $('[aria-label*="screen"], [aria-label*="Screen"]');
  }

  /**
   * End call button.
   */
  get endCallBtn() {
    return $('[aria-label="End call"]');
  }

  /**
   * Call status indicator (dot).
   */
  get statusIndicator() {
    return $('.rounded-full.w-3.h-3, .rounded-full.w-2.h-2');
  }

  /**
   * Participant grid container.
   */
  get participantGrid() {
    return $('.participant-grid');
  }

  /**
   * Individual participant tiles.
   */
  get participantTiles() {
    return $$('.participant-tile, .participant-video');
  }

  /**
   * Call duration display.
   */
  get duration() {
    return $('[aria-label="Call duration"]');
  }

  /**
   * Participant count display.
   */
  get participantCount() {
    return $('span*=👥');
  }

  /**
   * Entity/call name heading.
   */
  get callName() {
    return $('h1');
  }

  /**
   * Connecting state indicator.
   */
  get connectingState() {
    return $('[aria-label="Connecting to call"]');
  }

  /**
   * Reconnecting state indicator.
   */
  get reconnectingState() {
    return $('*=Reconnecting...');
  }

  /**
   * No active call state.
   */
  get idleState() {
    return $('*=No active call');
  }

  /**
   * Waiting for others state.
   */
  get waitingState() {
    return $('*=Waiting for others');
  }

  /**
   * Listen-only mode banner.
   */
  get listenOnlyBanner() {
    return $('*=Listen-only mode');
  }

  /**
   * Screen sharing indicator.
   */
  get screenSharingIndicator() {
    return $('*=sharing your screen');
  }

  /**
   * Media error banner.
   */
  get mediaErrorBanner() {
    return $('.media-error-banner, [role="alert"]');
  }

  /**
   * Call lobby container.
   */
  get callLobby() {
    return $('.call-lobby');
  }

  /**
   * Join call button in lobby.
   */
  get joinCallBtn() {
    return $('button=Join Call');
  }

  /**
   * Start call button.
   */
  get startCallBtn() {
    return $('button*=Start');
  }

  /**
   * Call notification popup.
   */
  get callNotification() {
    return $('.call-notification');
  }

  /**
   * Wait for call view to load.
   * @param {number} timeout - Maximum wait time in ms
   */
  async waitForLoad(timeout = 10000) {
    const container = await this.container;
    await container.waitForExist({ timeout });
  }

  /**
   * Navigate to call route.
   * @param {string} entityId - Entity for the call (default 'test')
   */
  async navigate(entityId = 'test') {
    await browser.url(`tauri://localhost/entity/${entityId}/call`);
    await this.waitForLoad();
  }

  /**
   * Check if in an active call.
   * @returns {Promise<boolean>}
   */
  async isInCall() {
    const controls = await this.controlsToolbar;
    if (!(await controls.isExisting())) {
      return false;
    }

    // Check if end call button is enabled
    const endBtn = await this.endCallBtn;
    if (await endBtn.isExisting()) {
      return !(await endBtn.getAttribute('disabled'));
    }
    return false;
  }

  /**
   * Check if muted.
   * @returns {Promise<boolean>}
   */
  async isMuted() {
    const muteBtn = await this.muteBtn;
    if (await muteBtn.isExisting()) {
      const pressed = await muteBtn.getAttribute('aria-pressed');
      return pressed === 'true';
    }
    return false;
  }

  /**
   * Check if video is enabled.
   * @returns {Promise<boolean>}
   */
  async isVideoEnabled() {
    const videoBtn = await this.videoBtn;
    if (await videoBtn.isExisting()) {
      const pressed = await videoBtn.getAttribute('aria-pressed');
      return pressed === 'true';
    }
    return false;
  }

  /**
   * Check if screen sharing.
   * @returns {Promise<boolean>}
   */
  async isScreenSharing() {
    const screenBtn = await this.screenShareBtn;
    if (await screenBtn.isExisting()) {
      const pressed = await screenBtn.getAttribute('aria-pressed');
      return pressed === 'true';
    }
    return false;
  }

  /**
   * Toggle mute.
   */
  async toggleMute() {
    const btn = await this.muteBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * Toggle video.
   */
  async toggleVideo() {
    const btn = await this.videoBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * Toggle screen share.
   */
  async toggleScreenShare() {
    const btn = await this.screenShareBtn;
    await btn.click();
    await browser.pause(300);
  }

  /**
   * End the current call.
   */
  async endCall() {
    const btn = await this.endCallBtn;
    await btn.click();
    await browser.pause(500);
  }

  /**
   * Get the call duration text.
   * @returns {Promise<string>}
   */
  async getDuration() {
    const duration = await this.duration;
    if (await duration.isExisting()) {
      return duration.getText();
    }
    return '00:00';
  }

  /**
   * Get the call name.
   * @returns {Promise<string>}
   */
  async getCallName() {
    const name = await this.callName;
    if (await name.isExisting()) {
      return name.getText();
    }
    return '';
  }

  /**
   * Get number of participant tiles visible.
   * @returns {Promise<number>}
   */
  async getParticipantCount() {
    const tiles = await this.participantTiles;
    return tiles.length;
  }

  /**
   * Check if connecting.
   * @returns {Promise<boolean>}
   */
  async isConnecting() {
    const connecting = await this.connectingState;
    return connecting.isExisting();
  }

  /**
   * Check if idle (no active call).
   * @returns {Promise<boolean>}
   */
  async isIdle() {
    const idle = await this.idleState;
    return idle.isExisting();
  }

  /**
   * Check if mini call view is visible.
   * @returns {Promise<boolean>}
   */
  async isMiniViewVisible() {
    const mini = await this.miniCallView;
    return mini.isDisplayed();
  }

  /**
   * Expand mini call view to full view.
   */
  async expandMiniView() {
    const expandBtn = await this.miniCallView.$('[aria-label="Expand call view"]');
    if (await expandBtn.isExisting()) {
      await expandBtn.click();
      await browser.pause(300);
    }
  }

  /**
   * Check if there are media errors.
   * @returns {Promise<boolean>}
   */
  async hasMediaErrors() {
    const banner = await this.mediaErrorBanner;
    return banner.isExisting();
  }

  /**
   * Check if in listen-only mode.
   * @returns {Promise<boolean>}
   */
  async isListenOnly() {
    const banner = await this.listenOnlyBanner;
    return banner.isExisting();
  }
}

export default new CallPage();
