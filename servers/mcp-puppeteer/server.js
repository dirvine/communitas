#!/usr/bin/env node
// Minimal MCP server exposing browser automation (Puppeteer)
// and Communitas app helpers via window.* test utilities.

import puppeteer from 'puppeteer';
import { z } from 'zod';
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';

const DEFAULT_URL = process.env.MCP_BROWSER_URL || 'http://localhost:1420';
const HEADLESS = process.env.MCP_BROWSER_HEADLESS === '1';

let browser = null;
let page = null;

async function ensureBrowser() {
  if (!browser) {
    const executablePath = process.env.PUPPETEER_EXECUTABLE_PATH || undefined;
    const launchOpts = {
      headless: HEADLESS,
      defaultViewport: { width: 1400, height: 900 },
      args: ['--no-sandbox', '--disable-dev-shm-usage'],
      executablePath,
    };
    console.log('[mcp-stdio] Launching Puppeteer', { headless: HEADLESS, executablePath: executablePath ? 'custom' : 'default' });
    browser = await puppeteer.launch(launchOpts);
  }
  if (!page) {
    page = await browser.newPage();
    page.on('console', msg => {
      const type = msg.type();
      if (type === 'error') {
        console.error('[browser:error]', msg.text());
      } else {
        console.log(`[browser:${type}]`, msg.text());
      }
    });
  }
  return { browser, page };
}

const server = new McpServer({ name: 'communitas-mcp-puppeteer', version: '0.1.0' });

// browser_navigate
server.registerTool(
  'browser_navigate',
  {
    title: 'Navigate to URL',
    description: 'Navigate the browser to a URL',
    inputSchema: { url: z.string().url().describe('Destination URL') },
  },
  async ({ url }) => {
    const { page } = await ensureBrowser();
    await page.goto(url, { waitUntil: 'networkidle2' });
    const title = await page.title();
    return { content: [{ type: 'text', text: `Navigated to ${url}\nTitle: ${title}` }] };
  }
);

// browser_wait_for
server.registerTool(
  'browser_wait_for',
  {
    title: 'Wait for selector',
    description: 'Wait until a selector appears',
    inputSchema: {
      selector: z.string().describe('CSS selector'),
      timeoutMs: z.number().int().positive().optional().describe('Timeout in ms (default 10000)'),
    },
  },
  async ({ selector, timeoutMs }) => {
    const { page } = await ensureBrowser();
    await page.waitForSelector(selector, { timeout: timeoutMs ?? 10000 });
    return { content: [{ type: 'text', text: `Selector present: ${selector}` }] };
  }
);

// browser_click
server.registerTool(
  'browser_click',
  {
    title: 'Click element',
    description: 'Click an element by CSS selector',
    inputSchema: { selector: z.string().describe('CSS selector to click') },
  },
  async ({ selector }) => {
    const { page } = await ensureBrowser();
    await page.click(selector, { delay: 10 });
    return { content: [{ type: 'text', text: `Clicked ${selector}` }] };
  }
);

// browser_click_text
server.registerTool(
  'browser_click_text',
  {
    title: 'Click element by text',
    description: 'Find a <button> or clickable element by textContent and click it',
    inputSchema: { text: z.string().min(1).describe('Text to match (includes)') },
  },
  async ({ text }) => {
    const { page } = await ensureBrowser();
    const found = await page.evaluateHandle((t) => {
      const els = Array.from(document.querySelectorAll('button, [role="button"], a, *'));
      return els.find((el) => (el.textContent || '').includes(t));
    }, text);
    const exists = await found.evaluate((el) => !!el);
    if (!exists) {
      return { content: [{ type: 'text', text: `No element with text including: ${text}` }] };
    }
    await found.click();
    return { content: [{ type: 'text', text: `Clicked element containing text: ${text}` }] };
  }
);

server.registerTool(
  'browser_wait_text',
  {
    title: 'Wait for text',
    description: 'Wait until given text appears',
    inputSchema: { text: z.string().min(1), timeoutMs: z.number().int().positive().optional() },
  },
  async ({ text, timeoutMs }) => {
    const { page } = await ensureBrowser();
    await page.waitForFunction(
      (t) => document.body && document.body.innerText && document.body.innerText.includes(t),
      { timeout: timeoutMs ?? 8000 },
      text,
    );
    return { content: [{ type: 'text', text: `Text present: ${text}` }] };
  }
);

server.registerTool(
  'browser_fill',
  {
    title: 'Fill input',
    description: 'Type into input/textarea matched by selector',
    inputSchema: { selector: z.string(), value: z.string() },
  },
  async ({ selector, value }) => {
    const { page } = await ensureBrowser();
    await page.focus(selector);
    await page.evaluate((sel) => {
      const el = document.querySelector(sel);
      if (el) (el).value = '';
    }, selector);
    await page.type(selector, value);
    return { content: [{ type: 'text', text: `Filled ${selector}` }] };
  }
);

server.registerTool(
  'browser_query_all_texts',
  {
    title: 'Query all texts',
    description: 'Return textContent array for selector',
    inputSchema: { selector: z.string() },
  },
  async ({ selector }) => {
    const { page } = await ensureBrowser();
    const texts = await page.$$eval(selector, els => els.map(el => (el.textContent || '').trim()).filter(Boolean));
    return { content: [{ type: 'text', text: JSON.stringify(texts) }] };
  }
);

server.registerTool(
  'browser_fill_by_label',
  {
    title: 'Fill input by label',
    description: 'Find input/textarea associated with label containing text and fill',
    inputSchema: { label: z.string().min(1), value: z.string() },
  },
  async ({ label, value }) => {
    const { page } = await ensureBrowser();
    const ok = await page.evaluate((lab, val) => {
      function findByLabelText(text) {
        const labels = Array.from(document.querySelectorAll('label'));
        const match = labels.find(l => (l.textContent || '').trim().toLowerCase().includes(text.toLowerCase()));
        if (!match) return false;
        let el = null;
        const forId = match.getAttribute('for');
        if (forId) el = document.getElementById(forId);
        if (!el) el = match.querySelector('input,textarea');
        if (!el) {
          const cand = match.parentElement?.querySelector('input,textarea');
          if (cand) el = cand;
        }
        if (el) {
          el.focus();
          el.value = '';
          el.dispatchEvent(new Event('input', { bubbles: true }));
          el.value = val;
          el.dispatchEvent(new Event('input', { bubbles: true }));
          return true;
        }
        return false;
      }
      return findByLabelText(lab);
    }, label, value);
    if (!ok) return { content: [{ type: 'text', text: `No input for label: ${label}` }] };
    return { content: [{ type: 'text', text: `Filled by label: ${label}` }] };
  }
);

// browser_type
server.registerTool(
  'browser_type',
  {
    title: 'Type into element',
    description: 'Type text into an input/textarea matched by CSS selector',
    inputSchema: { selector: z.string(), text: z.string() },
  },
  async ({ selector, text }) => {
    const { page } = await ensureBrowser();
    await page.focus(selector);
    await page.type(selector, text);
    return { content: [{ type: 'text', text: `Typed into ${selector}: ${text}` }] };
  }
);

// browser_snapshot
server.registerTool(
  'browser_snapshot',
  {
    title: 'Take screenshot',
    description: 'Capture a screenshot and return as base64',
    inputSchema: { fullPage: z.boolean().optional() },
  },
  async ({ fullPage }) => {
    const { page } = await ensureBrowser();
    const buf = await page.screenshot({ fullPage: !!fullPage, type: 'png' });
    const b64 = Buffer.from(buf).toString('base64');
    return {
      content: [
        { type: 'text', text: 'Screenshot captured (base64 PNG)' },
        { type: 'image', data: b64, mimeType: 'image/png' },
      ],
    };
  }
);

// browser_eval
server.registerTool(
  'browser_eval',
  {
    title: 'Evaluate JavaScript',
    description: 'Run JS in the page context and return the result as text',
    inputSchema: { script: z.string().describe('Function body or expression to eval') },
  },
  async ({ script }) => {
    const { page } = await ensureBrowser();
    const result = await page.evaluate((code) => {
      // eslint-disable-next-line no-new-func
      const fn = new Function(`return (${code});`);
      const val = fn();
      return typeof val === 'string' ? val : JSON.stringify(val);
    }, script);
    return { content: [{ type: 'text', text: String(result) }] };
  }
);

// app_* helpers that leverage Communitas window test utilities
async function ensureAppNavigated() {
  const { page } = await ensureBrowser();
  if (page.url() === 'about:blank') {
    await page.goto(DEFAULT_URL, { waitUntil: 'networkidle2' });
  }
  return page;
}

const appTools = [
  {
    name: 'app_test_identity',
    desc: 'Run window.testIdentity.testFlow() in Communitas',
    eval: () => (window.testIdentity?.testFlow ? window.testIdentity.testFlow() : 'testIdentity not available'),
  },
  {
    name: 'app_setup_workspace',
    desc: 'Run window.workspace.setup() in Communitas',
    eval: () => (window.workspace?.setup ? window.workspace.setup() : 'workspace not available'),
  },
  {
    name: 'app_test_groups',
    desc: 'Run window.tauriGroups.test() in Communitas',
    eval: () => (window.tauriGroups?.test ? window.tauriGroups.test() : 'tauriGroups not available'),
  },
  {
    name: 'app_test_group_messaging',
    desc: 'Run window.tauriGroups.testMessaging() in Communitas',
    eval: () => (window.tauriGroups?.testMessaging ? window.tauriGroups.testMessaging() : 'tauriGroups not available'),
  },
  {
    name: 'app_offline_simulate',
    desc: 'Run window.offlineTest.simulate() in Communitas',
    eval: () => (window.offlineTest?.simulate ? window.offlineTest.simulate() : 'offlineTest not available'),
  },
  {
    name: 'app_list_groups',
    desc: 'Run window.tauriGroups.listGroups() in Communitas',
    eval: () => (window.tauriGroups?.listGroups ? window.tauriGroups.listGroups() : 'tauriGroups not available'),
  },
  {
    name: 'app_offline_stats',
    desc: 'Get window.offlineTest.storage.getStats() in Communitas',
    eval: () => (window.offlineTest?.storage?.getStats ? window.offlineTest.storage.getStats() : 'offlineTest not available'),
  },
  {
    name: 'app_click_tab',
    desc: 'Click a top-level tab by visible text',
    // param passed at call time via page.evaluate arguments
    eval: (tabName) => {
      const btns = Array.from(document.querySelectorAll('button'));
      const el = btns.find(b => (b.textContent || '').trim().includes(tabName));
      if (el) { el.click(); return `clicked:${tabName}`; }
      return `not_found:${tabName}`;
    },
  },
  {
    name: 'app_offline_full_flow',
    desc: 'Run full offline flow: workspace.setup + offlineTest.test + list',
    eval: async () => {
      const ws = (window).workspace;
      const ot = (window).offlineTest;
      if (!ws?.setup || !ot?.test) return 'offline utilities not available';
      const setup = await ws.setup();
      const test = await ot.test();
      const list = await ws.list();
      return { setup, test, listed: true };
    }
  },
];

for (const tool of appTools) {
  const schema = tool.name === 'app_click_tab'
    ? { tabName: z.string().min(1) }
    : {};
  server.registerTool(
    tool.name,
    {
      title: tool.name,
      description: tool.desc,
      inputSchema: schema,
    },
    async (args) => {
      const page = await ensureAppNavigated();
      const result = tool.name === 'app_click_tab'
        ? await page.evaluate(tool.eval, args.tabName)
        : await page.evaluate(tool.eval);
      const text = typeof result === 'string' ? result : JSON.stringify(result);
      return { content: [{ type: 'text', text }] };
    }
  );
}

// Start server over stdio
const transport = new StdioServerTransport();
await server.connect(transport);

process.on('SIGINT', async () => {
  if (browser) await browser.close().catch(() => {});
  process.exit(0);
});
