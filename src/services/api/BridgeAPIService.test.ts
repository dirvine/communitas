// Manual test functions for BridgeAPIService
// Can be called from browser console or Chrome DevTools MCP
import { bridgeAPI } from './BridgeAPIService';
import { logger } from '../LoggingService';

/**
 * Test 1: Health check - verify bridge server is running
 */
export async function testBridgeHealth(): Promise<boolean> {
  logger.info('Testing bridge health check...');
  try {
    const isHealthy = await bridgeAPI.healthCheck();
    logger.info('Bridge health check result:', { isHealthy });
    return isHealthy;
  } catch (error) {
    logger.error('Bridge health check failed', { error });
    return false;
  }
}

/**
 * Test 2: Initialize core with test identity
 */
export async function testBridgeInitialize(): Promise<boolean> {
  logger.info('Testing bridge initialization...');
  try {
    await bridgeAPI.initialize(
      'ocean-forest-moon-star',
      'Test User',
      'Browser Test Device'
    );

    const status = await bridgeAPI.getStatus();
    logger.info('Bridge initialization result:', { status });
    return status.initialized;
  } catch (error) {
    logger.error('Bridge initialization failed', { error });
    return false;
  }
}

/**
 * Test 3: Create a test channel
 */
export async function testBridgeCreateChannel(): Promise<string | null> {
  logger.info('Testing bridge channel creation...');
  try {
    const channel = await bridgeAPI.createChannel(
      'Test Channel',
      'Channel created from browser test'
    );
    logger.info('Bridge channel creation result:', { channel });
    return channel.id;
  } catch (error) {
    logger.error('Bridge channel creation failed', { error });
    return null;
  }
}

/**
 * Test 4: List all channels
 */
export async function testBridgeListChannels(): Promise<number> {
  logger.info('Testing bridge channel listing...');
  try {
    const result = await bridgeAPI.listChannels();
    logger.info('Bridge channel listing result:', {
      count: result.channels.length,
      channels: result.channels
    });
    return result.channels.length;
  } catch (error) {
    logger.error('Bridge channel listing failed', { error });
    return 0;
  }
}

/**
 * Test 5: Send message to channel (requires valid channel ID and recipients)
 */
export async function testBridgeSendMessage(
  channelId: string,
  recipients: string[]
): Promise<boolean> {
  logger.info('Testing bridge message sending...');
  try {
    const result = await bridgeAPI.sendChannelMessage(
      channelId,
      'Test message from browser',
      recipients
    );
    logger.info('Bridge message sending result:', { result });
    return result.success;
  } catch (error) {
    logger.error('Bridge message sending failed', { error });
    return false;
  }
}

/**
 * Run all bridge tests in sequence
 */
export async function runAllBridgeTests(): Promise<void> {
  logger.info('Starting comprehensive bridge test suite...');

  // Test 1: Health check
  const isHealthy = await testBridgeHealth();
  if (!isHealthy) {
    logger.error('Bridge server not healthy - aborting tests');
    return;
  }

  // Test 2: Initialize
  const isInitialized = await testBridgeInitialize();
  if (!isInitialized) {
    logger.error('Bridge initialization failed - aborting tests');
    return;
  }

  // Test 3: Create channel
  const channelId = await testBridgeCreateChannel();
  if (!channelId) {
    logger.error('Bridge channel creation failed - aborting tests');
    return;
  }

  // Test 4: List channels
  const channelCount = await testBridgeListChannels();
  logger.info('Channel count after creation:', { channelCount });

  // Test 5: Send message (requires valid recipients)
  const messageSent = await testBridgeSendMessage(
    channelId,
    ['ocean-forest-moon-star'] // Test identity
  );
  logger.info('Message sent:', { messageSent });

  logger.info('Bridge test suite completed!');
}

// Make tests available globally for console/MCP access
if (typeof window !== 'undefined') {
  (window as any).bridgeTests = {
    health: testBridgeHealth,
    initialize: testBridgeInitialize,
    createChannel: testBridgeCreateChannel,
    listChannels: testBridgeListChannels,
    sendMessage: testBridgeSendMessage,
    runAll: runAllBridgeTests,
  };
}
