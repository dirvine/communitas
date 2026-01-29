#!/usr/bin/env node

/**
 * E2E Test Coverage Report Generator
 *
 * Analyzes E2E test files and generates a coverage summary
 * showing test count per widget.
 */

const fs = require('fs');
const path = require('path');

// Widget test files
const widgetTests = {
  'Contacts': 'contacts.spec.js',
  'Messages': 'messages.spec.js',
  'Kanban': 'kanban.spec.js',
  'Drive': 'drive.spec.js',
  'Canvas': 'canvas.spec.js',
  'Settings': 'settings.spec.js',
  'Search': 'search.spec.js',
  'Notifications': 'notifications.spec.js'
};

const e2eDir = __dirname;

// Count tests in a file
function countTests(filePath) {
  try {
    const content = fs.readFileSync(path.join(e2eDir, filePath), 'utf8');
    const matches = content.match(/test\(/g);
    return matches ? matches.length : 0;
  } catch (err) {
    return 0;
  }
}

// Generate report
console.log('\n═══════════════════════════════════════════════');
console.log('   Widget E2E Test Coverage Report');
console.log('═══════════════════════════════════════════════\n');

let totalTests = 0;
let widgetsCovered = 0;

console.log('Widget                Tests      Status');
console.log('───────────────────────────────────────────────');

for (const [widget, filename] of Object.entries(widgetTests)) {
  const testCount = countTests(filename);
  const status = testCount >= 8 ? '✅' : testCount > 0 ? '⚠️' : '❌';

  console.log(`${widget.padEnd(20)} ${String(testCount).padStart(5)}      ${status}`);

  totalTests += testCount;
  if (testCount > 0) widgetsCovered++;
}

console.log('───────────────────────────────────────────────');

// Integration tests
const integrationTests = countTests('integration.spec.js');
const smokeTests = countTests('smoke.spec.js');

console.log(`${'Integration'.padEnd(20)} ${String(integrationTests).padStart(5)}      ${integrationTests > 0 ? '✅' : '❌'}`);
console.log(`${'Infrastructure'.padEnd(20)} ${String(smokeTests).padStart(5)}      ${smokeTests > 0 ? '✅' : '❌'}`);

totalTests += integrationTests + smokeTests;

console.log('═══════════════════════════════════════════════');
console.log(`Total Widgets: ${Object.keys(widgetTests).length}`);
console.log(`Widgets Covered: ${widgetsCovered}/${Object.keys(widgetTests).length} (${Math.round(widgetsCovered / Object.keys(widgetTests).length * 100)}%)`);
console.log(`Total Test Cases: ${totalTests}`);
console.log('═══════════════════════════════════════════════\n');

// Exit with appropriate code
const allCovered = widgetsCovered === Object.keys(widgetTests).length;
process.exit(allCovered ? 0 : 1);
