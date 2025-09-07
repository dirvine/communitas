import http from 'http';
import { chromium } from 'playwright';
import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { HTTPServerTransport } from '@modelcontextprotocol/sdk/server/transport/http.js';

const APP_URL = process.env.MCP_BROWSER_URL || 'http://localhost:1420';
const HOST = process.env.MCP_HTTP_HOST || '127.0.0.1';
const PORT = Number(process.env.MCP_HTTP_PORT || 8899);

const browser = await chromium.launch({ headless: process.env.MCP_BROWSER_HEADLESS !== '0' });
const context = await browser.newContext();
const page = await context.newPage();

const mcp = new Server({ name: 'mcp-playwright', version: '0.1.0' });

mcp.tool('browser_navigate', {
  description: 'Navigate to URL and return title',
  inputSchema: { type: 'object', properties: { url: { type: 'string' } }, required: ['url'] },
  execute: async ({ url }) => {
    await page.goto(url, { waitUntil: 'domcontentloaded' });
    return { content: [{ type: 'text', text: await page.title() }] };
  },
});

mcp.tool('browser_wait_for', {
  description: 'Wait for selector',
  inputSchema: { type: 'object', properties: { selector: { type: 'string' }, timeout: { type: 'number' } }, required: ['selector'] },
  execute: async ({ selector, timeout = 10000 }) => {
    await page.waitForSelector(selector, { timeout });
    return { content: [{ type: 'text', text: 'ok' }] };
  },
});

mcp.tool('browser_click_text', {
  description: 'Click first element containing text',
  inputSchema: { type: 'object', properties: { text: { type: 'string' } }, required: ['text'] },
  execute: async ({ text }) => {
    await page.getByText(text, { exact: false }).first().click();
    return { content: [{ type: 'text', text: 'clicked' }] };
  },
});

mcp.tool('browser_eval', {
  description: 'Evaluate JS and return JSON stringified result',
  inputSchema: { type: 'object', properties: { code: { type: 'string' } }, required: ['code'] },
  execute: async ({ code }) => {
    const result = await page.evaluate(new Function(code));
    return { content: [{ type: 'text', text: JSON.stringify(result) }] };
  },
});

mcp.tool('browser_screenshot', {
  description: 'PNG screenshot as base64',
  inputSchema: { type: 'object', properties: { fullPage: { type: 'boolean' } } },
  execute: async ({ fullPage = true }) => {
    const buf = await page.screenshot({ fullPage });
    return { content: [{ type: 'bytes', mimeType: 'image/png', data: buf.toString('base64') }] };
  },
});

mcp.tool('browser_query_all_texts', {
  description: 'Return an array of visible texts matching a substring (case-insensitive)',
  inputSchema: { type: 'object', properties: { text: { type: 'string' } }, required: ['text'] },
  execute: async ({ text }) => {
    const result = await page.evaluate((needle) => {
      const isVisible = (el) => {
        const s = getComputedStyle(el);
        return s && s.display !== 'none' && s.visibility !== 'hidden' && +s.opacity !== 0;
      };
      const walk = (root) => {
        const out = [];
        const it = document.createNodeIterator(root, NodeFilter.SHOW_TEXT);
        let n;
        while ((n = it.nextNode())) {
          const t = n.nodeValue.trim();
          if (!t) continue;
          const el = n.parentElement;
          if (el && isVisible(el)) out.push(t);
        }
        return out;
      };
      return walk(document.body).filter(t => t.toLowerCase().includes(needle.toLowerCase()));
    }, text);
    return { content: [{ type: 'text', text: JSON.stringify(result) }] };
  },
});

await page.goto(APP_URL, { waitUntil: 'domcontentloaded' });

const httpServer = http.createServer();
const transport = new HTTPServerTransport({ server: httpServer });
await mcp.connect(transport);
httpServer.listen(PORT, HOST, () => {
  console.log(`MCP HTTP server listening at http://${HOST}:${PORT}`);
});
