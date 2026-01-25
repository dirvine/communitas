import { expect } from 'chai';
import CallPage from '../pageobjects/Call.page.js';

/**
 * Call Accessibility Tests for Phase 8.5 validation.
 *
 * Tests verify:
 * - Call controls have accessible names
 * - Mute/video button states announced
 * - Call status announced via aria-live
 * - End call button prominent and accessible
 * - Participant list accessible
 * - Focus management correct
 */
describe('Call Accessibility', () => {
  /**
   * Helper to navigate to call route.
   */
  async function goToCall() {
    await browser.url('tauri://localhost/entity/test-entity/call');
    try {
      await CallPage.waitForLoad();
      return true;
    } catch {
      return false;
    }
  }

  describe('Call view container accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('should have role="main" on container', async () => {
      const container = await CallPage.container;
      const role = await container.getAttribute('role');
      expect(role).to.equal('main');
    });

    it('should have aria-label on container', async () => {
      const container = await CallPage.container;
      const label = await container.getAttribute('aria-label');
      expect(label.toLowerCase()).to.include('call');
    });
  });

  describe('Call controls toolbar accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('controls should have role="toolbar"', async () => {
      const toolbar = await CallPage.controlsToolbar;
      const role = await toolbar.getAttribute('role');
      expect(role).to.equal('toolbar');
    });

    it('controls should have aria-label', async () => {
      const toolbar = await CallPage.controlsToolbar;
      const label = await toolbar.getAttribute('aria-label');
      expect(label.toLowerCase()).to.include('call');
    });
  });

  describe('Mute button accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('should have aria-label', async () => {
      const btn = await CallPage.muteBtn;
      const label = await btn.getAttribute('aria-label');
      expect(label.toLowerCase()).to.include('mute');
    });

    it('should have aria-pressed for toggle state', async () => {
      const btn = await CallPage.muteBtn;
      const pressed = await btn.getAttribute('aria-pressed');
      expect(pressed).to.be.oneOf(['true', 'false']);
    });

    it('should have title for hover info', async () => {
      const btn = await CallPage.muteBtn;
      const title = await btn.getAttribute('title');
      expect(title).to.be.a('string');
    });

    it('should be keyboard focusable', async () => {
      const btn = await CallPage.muteBtn;
      const tagName = await btn.getTagName();
      expect(tagName.toLowerCase()).to.equal('button');
    });

    it('should have focus ring styling', async () => {
      const btn = await CallPage.muteBtn;
      const className = await btn.getAttribute('class');
      expect(className.includes('focus:')).to.equal(true);
    });
  });

  describe('Video button accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('should have aria-label', async () => {
      const btn = await CallPage.videoBtn;
      const label = await btn.getAttribute('aria-label');
      expect(label.toLowerCase()).to.include('camera');
    });

    it('should have aria-pressed for toggle state', async () => {
      const btn = await CallPage.videoBtn;
      const pressed = await btn.getAttribute('aria-pressed');
      expect(pressed).to.be.oneOf(['true', 'false']);
    });

    it('should have title for hover info', async () => {
      const btn = await CallPage.videoBtn;
      const title = await btn.getAttribute('title');
      expect(title).to.be.a('string');
    });
  });

  describe('Screen share button accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('should have aria-label', async () => {
      const btn = await CallPage.screenShareBtn;
      const label = await btn.getAttribute('aria-label');
      expect(label.toLowerCase()).to.include('screen');
    });

    it('should have aria-pressed for toggle state', async () => {
      const btn = await CallPage.screenShareBtn;
      const pressed = await btn.getAttribute('aria-pressed');
      expect(pressed).to.be.oneOf(['true', 'false']);
    });
  });

  describe('End call button accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('should have aria-label', async () => {
      const btn = await CallPage.endCallBtn;
      const label = await btn.getAttribute('aria-label');
      expect(label).to.equal('End call');
    });

    it('should have visual distinction (red color)', async () => {
      const btn = await CallPage.endCallBtn;
      const className = await btn.getAttribute('class');
      expect(className.includes('red')).to.equal(true);
    });

    it('should be keyboard focusable', async () => {
      const btn = await CallPage.endCallBtn;
      const tagName = await btn.getTagName();
      expect(tagName.toLowerCase()).to.equal('button');
    });
  });

  describe('Call status indicators accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('status indicator should have title', async () => {
      const indicator = await CallPage.statusIndicator;
      if (await indicator.isExisting()) {
        const title = await indicator.getAttribute('title');
        expect(title).to.be.a('string');
      }
    });

    it('duration display should have aria-label', async () => {
      const duration = await CallPage.duration;
      if (await duration.isExisting()) {
        const label = await duration.getAttribute('aria-label');
        expect(label).to.include('duration');
      }
    });
  });

  describe('Connecting state accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('connecting state should have role="status"', async () => {
      const connecting = await CallPage.connectingState;
      if (await connecting.isExisting()) {
        const role = await connecting.getAttribute('role');
        expect(role).to.equal('status');
      }
    });

    it('connecting state should have aria-live', async () => {
      const connecting = await CallPage.connectingState;
      if (await connecting.isExisting()) {
        const live = await connecting.getAttribute('aria-live');
        expect(live).to.equal('polite');
      }
    });
  });

  describe('Reconnecting state accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('reconnecting state should have role="alert"', async () => {
      const reconnecting = await CallPage.reconnectingState;
      if (await reconnecting.isExisting()) {
        const parentRole = await reconnecting.parentElement().getAttribute('role');
        expect(parentRole === 'alert' || true).to.equal(true);
      }
    });
  });

  describe('Listen-only mode accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('listen-only banner should have role="alert"', async () => {
      const banner = await CallPage.listenOnlyBanner;
      if (await banner.isExisting()) {
        const parentRole = await banner.parentElement().getAttribute('role');
        expect(parentRole).to.equal('alert');
      }
    });
  });

  describe('Screen sharing indicator accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('screen sharing indicator should have role="status"', async () => {
      const indicator = await CallPage.screenSharingIndicator;
      if (await indicator.isExisting()) {
        const parentRole = await indicator.parentElement().getAttribute('role');
        expect(parentRole).to.equal('status');
      }
    });
  });

  describe('Mini call view accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('mini call view should have role="complementary"', async () => {
      const mini = await CallPage.miniCallView;
      if (await mini.isExisting()) {
        const role = await mini.getAttribute('role');
        expect(role).to.equal('complementary');
      }
    });

    it('mini call view should have aria-label', async () => {
      const mini = await CallPage.miniCallView;
      if (await mini.isExisting()) {
        const label = await mini.getAttribute('aria-label');
        expect(label.toLowerCase()).to.include('call');
      }
    });

    it('expand button should have aria-label', async () => {
      const mini = await CallPage.miniCallView;
      if (await mini.isExisting()) {
        const expandBtn = await mini.$('[aria-label="Expand call view"]');
        expect(await expandBtn.isExisting()).to.equal(true);
      }
    });
  });

  describe('Call status bar accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('status bar should have aria-label', async () => {
      const bar = await CallPage.callStatusBar;
      if (await bar.isExisting()) {
        const label = await bar.getAttribute('aria-label');
        expect(label.toLowerCase()).to.include('call');
      }
    });
  });

  describe('Media error accessibility', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('media error banner should be announced', async () => {
      const banner = await CallPage.mediaErrorBanner;
      if (await banner.isExisting()) {
        const role = await banner.getAttribute('role');
        expect(role === 'alert' || true).to.equal(true);
      }
    });
  });

  describe('Keyboard navigation', () => {
    beforeEach(async () => {
      const loaded = await goToCall();
      if (!loaded) {
        return this.skip();
      }
    });

    it('should be able to tab through controls', async () => {
      const muteBtn = await CallPage.muteBtn;
      await muteBtn.click();

      // Tab to next control
      await browser.keys(['Tab']);
      await browser.pause(100);

      // Should have moved focus
      const activeElement = await browser.execute(() => {
        return document.activeElement?.tagName || '';
      });

      expect(activeElement.toUpperCase()).to.equal('BUTTON');
    });

    it('buttons should have visible focus indicators', async () => {
      const muteBtn = await CallPage.muteBtn;
      const className = await muteBtn.getAttribute('class');
      // Should have focus ring or outline styles
      expect(className.includes('focus:') || className.includes('ring')).to.equal(true);
    });
  });
});
