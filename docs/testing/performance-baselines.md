# MCP Apps Widget Performance Baselines

## Overview

This document establishes performance baselines for Communitas MCP Apps widgets. These metrics help ensure consistent user experience across different MCP hosts.

## Test Environment

### Reference Configuration

| Component | Specification |
|-----------|---------------|
| Browser Engine | Chromium 120+ (Claude Desktop) |
| Memory | 8GB RAM minimum |
| Network | Localhost (no network latency) |
| Mode | Demo mode (`--demo` flag) |

### Measurement Tools

- Browser DevTools Performance tab
- `performance.now()` timing in test harness
- Memory profiler for heap snapshots

## Widget Load Time Baselines

### Target Metrics

| Metric | Target | Acceptable | Poor |
|--------|--------|------------|------|
| First Contentful Paint (FCP) | <100ms | <200ms | >200ms |
| Time to Interactive (TTI) | <200ms | <400ms | >400ms |
| Largest Contentful Paint (LCP) | <150ms | <300ms | >300ms |
| Total Blocking Time (TBT) | <50ms | <100ms | >100ms |

### Individual Widget Baselines

#### Contacts Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <10ms | Inline bundle, no external loads |
| JS Execute | <30ms | MCP bridge initialization |
| Initial Render | <50ms | Empty contact list |
| Data Load | <100ms | Via MCP bridge callTool |
| Full Interactive | <150ms | Search ready, click handlers active |

**Test Scenario:**
1. Load widget from `ui://communitas/contacts`
2. MCP bridge initializes
3. `list_contacts` tool called
4. Contact list renders with demo data

#### Messages Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <10ms | |
| JS Execute | <30ms | |
| Initial Render | <60ms | Thread list skeleton |
| Thread Load | <100ms | First 20 threads |
| Message Load | <150ms | Selected thread messages |

**Test Scenario:**
1. Load widget
2. Fetch thread list
3. Select first thread
4. Render messages

#### Kanban Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <15ms | Larger HTML structure |
| JS Execute | <40ms | Drag-drop setup |
| Initial Render | <80ms | Board grid layout |
| Full Board | <200ms | All columns and cards |

**Test Scenario:**
1. Load widget
2. Fetch board data
3. Render columns and cards
4. Initialize drag-drop

#### Drive Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <10ms | |
| JS Execute | <30ms | |
| Initial Render | <50ms | File list skeleton |
| File List | <100ms | First 50 files |
| Preview Load | <200ms | Image/text preview |

**Test Scenario:**
1. Load widget
2. Fetch root directory
3. Render file list
4. Select file for preview

#### Canvas Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <10ms | |
| JS Execute | <50ms | Canvas/SVG setup |
| Initial Render | <100ms | Empty canvas |
| Snapshot Load | <300ms | Complex canvas state |

**Test Scenario:**
1. Load widget
2. Initialize canvas element
3. Fetch canvas snapshot
4. Render elements and layers

#### Settings Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <10ms | |
| JS Execute | <20ms | Simpler initialization |
| Initial Render | <40ms | Settings form |
| Preferences Load | <80ms | User settings data |

#### Search Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <10ms | |
| JS Execute | <20ms | |
| Initial Render | <40ms | Search input ready |
| Query Response | <200ms | First results appear |

#### Notifications Widget

| Metric | Baseline | Notes |
|--------|----------|-------|
| HTML Parse | <10ms | |
| JS Execute | <20ms | |
| Initial Render | <40ms | |
| Notification Load | <100ms | First 20 notifications |

## MCP Bridge Performance

### postMessage Latency

| Operation | Target | Notes |
|-----------|--------|-------|
| Message Send | <1ms | To MCP host |
| Message Receive | <1ms | From MCP host |
| Round Trip | <10ms | Complete request/response |

### Tool Call Performance

| Tool | Target | Notes |
|------|--------|-------|
| list_contacts | <50ms | Demo data |
| list_threads | <50ms | Demo data |
| list_files | <50ms | Demo data |
| send_message | <30ms | Async acknowledgment |
| search | <100ms | Full-text search |

## Memory Baselines

### Widget Memory Usage

| Widget | Initial | Active | Max |
|--------|---------|--------|-----|
| Contacts | 2MB | 5MB | 10MB |
| Messages | 3MB | 8MB | 20MB |
| Kanban | 4MB | 10MB | 25MB |
| Drive | 3MB | 10MB | 30MB |
| Canvas | 5MB | 15MB | 50MB |
| Settings | 1MB | 2MB | 5MB |
| Search | 2MB | 5MB | 15MB |
| Notifications | 2MB | 4MB | 10MB |

### Memory Leak Prevention

- No detached DOM nodes after widget close
- Event listeners properly removed on cleanup
- MCP bridge handlers unsubscribed
- Periodic garbage collection verification

## Network Performance

### Bundle Size Targets

| Component | Target | Max |
|-----------|--------|-----|
| Widget HTML (each) | <50KB | 100KB |
| Shared styles.css | <10KB | 20KB |
| MCP bridge.js | <15KB | 30KB |
| Total per widget | <75KB | 150KB |

### Caching Strategy

| Resource | Cache Duration | Notes |
|----------|----------------|-------|
| Widget HTML | Session | Embedded in binary |
| Static assets | 1 hour | Shared styles/scripts |
| Tool responses | None | Always fresh |

## Rendering Performance

### Frame Rate Targets

| Interaction | Target FPS | Notes |
|-------------|------------|-------|
| Scroll | 60fps | Smooth list scrolling |
| Drag | 60fps | Kanban card drag |
| Animation | 60fps | Transitions, loaders |
| Input | 60fps | No input lag |

### Long Task Prevention

- No single JS task >50ms
- Use requestAnimationFrame for animations
- Debounce search input (150ms)
- Virtualize long lists (>100 items)

## Test Procedures

### Automated Performance Tests

```javascript
// Performance timing test
async function measureWidgetLoad(widgetUri) {
    const start = performance.now();

    // Load widget
    const iframe = document.createElement('iframe');
    iframe.src = widgetUri;
    document.body.appendChild(iframe);

    await new Promise(resolve => iframe.onload = resolve);

    const loadTime = performance.now() - start;
    console.log(`Widget load: ${loadTime.toFixed(2)}ms`);

    // Measure interactive
    const bridge = iframe.contentWindow.bridge;
    const interactiveStart = performance.now();
    await bridge.callTool('list_contacts', {});
    const interactiveTime = performance.now() - interactiveStart;
    console.log(`Interactive: ${interactiveTime.toFixed(2)}ms`);

    return { loadTime, interactiveTime };
}
```

### Manual Performance Audit

1. Open Chrome DevTools > Performance
2. Start recording
3. Load widget via MCP tool call
4. Stop recording after widget interactive
5. Analyze:
   - Main thread activity
   - Scripting time
   - Rendering time
   - Painting time

### Memory Audit

1. Open Chrome DevTools > Memory
2. Take heap snapshot (baseline)
3. Load widget
4. Interact with widget
5. Take heap snapshot (active)
6. Close widget
7. Force GC and take snapshot (after)
8. Compare retained sizes

## Regression Testing

### Performance CI Check

```bash
# Run performance tests
npm run test:performance

# Acceptable thresholds
# - Load time regression: <10%
# - Memory regression: <20%
# - Frame rate: >55fps minimum
```

### Baseline Updates

Performance baselines should be updated when:

1. Major widget refactoring
2. New MCP host support
3. Significant new features
4. Quarterly review

## Known Performance Considerations

### MCP Host Variations

| Host | Notes |
|------|-------|
| Claude Desktop | Chromium-based, best performance |
| ChatGPT | Browser-dependent, may vary |
| VS Code | Webview panel, may have overhead |

### Optimization Opportunities

1. **Lazy loading** - Load data on demand
2. **Virtual scrolling** - For long lists
3. **Image optimization** - Thumbnails for previews
4. **Code splitting** - Load only needed modules
5. **Worker threads** - Offload heavy computation

## Monitoring

### Production Metrics

When deployed, monitor:

- P50/P95/P99 load times
- Error rates by widget
- Memory usage over time
- User interaction patterns

### Alerting Thresholds

| Metric | Warning | Critical |
|--------|---------|----------|
| Load time P95 | >500ms | >1000ms |
| Error rate | >1% | >5% |
| Memory leak | >50MB/hr | >100MB/hr |

---

*Last updated: 2026-01-27*
*Baseline version: 1.0*
