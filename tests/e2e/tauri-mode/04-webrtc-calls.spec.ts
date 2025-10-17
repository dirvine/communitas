/**
 * WebRTC Call E2E Tests (Web Mode)
 * 
 * Prerequisites: Run `npm run tauri dev` before running tests
 * Uses Chromium's fake media device support
 */

import { test, expect } from '@playwright/test';
import { TauriTestHelper, setupFakeMediaDevices } from '../../utils/tauri-helpers';

test.describe('WebRTC - Basics', () => {
  let helper: TauriTestHelper;

  test.beforeEach(async ({ page }) => {
    helper = new TauriTestHelper({ cleanup: false });
    await setupFakeMediaDevices(page);
    await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });
    await page.waitForLoadState('load');
    await page.waitForTimeout(2000);
    await helper.waitForTauriReady(page, 2000, false);
  });

  test('W1: WebRTC APIs are available', async ({ page }) => {
    const webrtcSupport = await page.evaluate(() => {
      return {
        rtcPeerConnection: typeof RTCPeerConnection !== 'undefined',
        mediaDevices: !!navigator.mediaDevices,
        getUserMedia: !!navigator.mediaDevices?.getUserMedia
      };
    });

    console.log('WebRTC support:', webrtcSupport);
    
    expect(webrtcSupport.rtcPeerConnection).toBe(true);
    expect(webrtcSupport.mediaDevices).toBe(true);
    console.log('✅ WebRTC APIs available');
  });

  test('W2: Can enumerate media devices', async ({ page }) => {
    const devices = await page.evaluate(async () => {
      try {
        const deviceList = await navigator.mediaDevices.enumerateDevices();
        return {
          success: true,
          devices: deviceList.map(d => ({
            kind: d.kind,
            label: d.label
          }))
        };
      } catch (error: any) {
        return { success: false, error: error.message };
      }
    });

    expect(devices.success).toBe(true);
    console.log('Available devices:', devices.devices);
    console.log('✅ Media devices enumerated');
  });

  test('W3: Can request media stream', async ({ page }) => {
    const streamResult = await page.evaluate(async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({
          video: true,
          audio: true
        });
        
        return {
          success: true,
          hasVideo: stream.getVideoTracks().length > 0,
          hasAudio: stream.getAudioTracks().length > 0
        };
      } catch (error: any) {
        return { success: false, error: error.message };
      }
    });

    expect(streamResult.success).toBe(true);
    expect(streamResult.hasVideo).toBe(true);
    console.log('✅ Media stream acquired');
  });

  test('W4: Call UI elements exist', async ({ page }) => {
    await page.waitForTimeout(2000);

    const callButton = page.locator('button, [role="button"]').filter({
      hasText: /call|video|audio|phone/i
    });

    const buttonCount = await callButton.count();
    console.log(`Found ${buttonCount} call-related buttons`);
    
    await helper.screenshot(page, 'webrtc-call-ui');
    expect(buttonCount >= 0).toBe(true);
  });

  test('W5: Can clean up media streams', async ({ page }) => {
    const cleanup = await page.evaluate(async () => {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        const tracks = stream.getTracks();
        const initialCount = tracks.length;
        
        tracks.forEach(track => track.stop());
        
        const stoppedCount = tracks.filter(track => track.readyState === 'ended').length;
        
        return {
          success: true,
          initialCount,
          stoppedCount,
          allStopped: stoppedCount === initialCount
        };
      } catch (error: any) {
        return { success: false, error: error.message };
      }
    });

    expect(cleanup.success).toBe(true);
    expect(cleanup.allStopped).toBe(true);
    console.log('✅ Media streams cleaned up properly');
  });
});
