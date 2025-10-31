# DNS-Free Website Publishing - UI Implementation Plan

**Date:** 2025-01-29  
**Status:** Comprehensive Design & Implementation Guide  
**Goal:** Production-ready Publisher Wizard + Viewer

---

## 🎯 Executive Summary

**We have excellent starting points:**
- ✅ `SitesDemo.tsx` - Basic publish/fetch UI
- ✅ `SitesService.ts` - Tauri IPC bindings
- ✅ `WebsitePanel.tsx` - Markdown editor for /website/ disk
- ✅ Material-UI components
- ✅ React + TypeScript stack

**What we need to build:**
- 🎨 4-Step Publisher Wizard (professional, guided experience)
- 🌐 Viewer with address bar (DNS-free browsing)
- 🔒 Security visibility (PQC lock, TOFU, fingerprints)
- 📊 Progress indicators (discovery, fetching, caching)
- ⚡ Offline-first UX (cache badges, offline mode)

**Estimated Effort:** 3-4 days

---

## 🏗️ ARCHITECTURE OVERVIEW

### Component Hierarchy

```
WebsitesSection (new main component)
├── PublisherWizard (Stepper with 4 steps)
│   ├── Step1_SelectContent
│   │   ├── FolderPicker
│   │   └── FileScanPreview
│   ├── Step2_NameAndKey
│   │   ├── FourWordsInput (shared)
│   │   ├── NameValidator
│   │   └── KeyManager
│   ├── Step3_SignReview
│   │   ├── ManifestPreview
│   │   ├── PQCSignButton
│   │   └── NameRecordSign
│   └── Step4_PublishSeed
│       ├── PublishProgress (shared)
│       └── ShareCard
│
├── Viewer (browser-like interface)
│   ├── ViewerToolbar
│   │   ├── FourWordsAddressBar
│   │   ├── NavigationButtons
│   │   ├── PQCLockIndicator (shared)
│   │   └── OfflineChip (shared)
│   ├── DiscoveryStatusBar
│   │   ├── ProviderCounter
│   │   └── FetchProgress (shared)
│   ├── ContentPanel
│   │   └── SandboxedWebView
│   └── SiteInfoDrawer
│       ├── IdentitySection
│       ├── SecuritySection
│       ├── ManifestSection
│       ├── ProvidersSection
│       └── CacheSection
│
└── ManagePublications (list view)
    ├── PublicationsList
    └── PublicationCard
```

### Shared Components (Reusable)

**Already Exist:**
- ✅ Material-UI (Button, TextField, Stepper, etc.)
- ✅ React hooks infrastructure

**To Create:**
1. `FourWordsInput.tsx` - Validated four-word input with autocomplete
2. `PQCLockIndicator.tsx` - Security status indicator
3. `OfflineChip.tsx` - Offline/cache status badge
4. `ProviderList.tsx` - Provider discovery UI
5. `FetchProgress.tsx` - Block fetching progress bar
6. `PublishProgress.tsx` - Publishing progress timeline
7. `SiteKeyManager.tsx` - Key generation/import/export

---

## 📝 DETAILED COMPONENT SPECIFICATIONS

### 1. PublisherWizard Component

**File:** `src/components/websites/PublisherWizard.tsx`

```typescript
interface PublisherWizardProps {
  entityId: string;
  fourWords: string;
  onComplete?: (siteId: string) => void;
  onCancel?: () => void;
}

interface PublishState {
  step: 1 | 2 | 3 | 4;
  
  // Step 1: Content selection
  selectedPath: string | null;
  scannedFiles: FileInfo[];
  totalBytes: number;
  estimatedBlocks: number;
  entryFile: string; // default: index.html
  excludePatterns: string[]; // default: ['.git', 'node_modules', '.*']
  
  // Step 2: Name & Key
  fourWordsName: string;
  nameValid: boolean;
  nameConflict: NameConflictInfo | null;
  siteKeyType: 'generate' | 'import';
  publicKey: Uint8Array | null;
  privateKey: Uint8Array | null; // Encrypted in memory
  keyFingerprint: string;
  
  // Step 3: Sign & Review
  manifestVersion: number;
  manifest: SiteManifest | null;
  manifestSigned: boolean;
  nameRecordSigned: boolean;
  
  // Step 4: Publish & Seed
  publishPhase: 'chunking' | 'caching' | 'signing' | 'announcing' | 'seeding' | 'done' | 'error';
  blocksPrepared: number;
  blocksTotal: number;
  providerCount: number;
  errors: PublishError[];
}

const PublisherWizard: React.FC<PublisherWizardProps> = ({ ... }) => {
  const [state, setState] = useState<PublishState>({
    step: 1,
    excludePatterns: ['.git', 'node_modules', '.*', '*.tmp'],
    entryFile: 'index.html',
    manifestVersion: 1,
    // ... defaults
  });
  
  // Event listeners for progress
  useEffect(() => {
    const unlistenChunk = listen('sites:publish_chunk_progress', (event) => {
      setState(s => ({ ...s, blocksPrepared: event.payload.blocksCreated }));
    });
    
    const unlistenStatus = listen('sites:publish_status', (event) => {
      setState(s => ({ ...s, publishPhase: event.payload.phase }));
    });
    
    return () => {
      unlistenChunk.then(fn => fn());
      unlistenStatus.then(fn => fn());
    };
  }, []);
  
  return (
    <Dialog open fullScreen>
      <DialogTitle>Publish Website</DialogTitle>
      <DialogContent>
        <Stepper activeStep={state.step - 1}>
          <Step><StepLabel>Select Content</StepLabel></Step>
          <Step><StepLabel>Name & Key</StepLabel></Step>
          <Step><StepLabel>Sign & Review</StepLabel></Step>
          <Step><StepLabel>Publish & Seed</StepLabel></Step>
        </Stepper>
        
        {state.step === 1 && <Step1_SelectContent state={state} setState={setState} />}
        {state.step === 2 && <Step2_NameAndKey state={state} setState={setState} />}
        {state.step === 3 && <Step3_SignReview state={state} setState={setState} />}
        {state.step === 4 && <Step4_PublishSeed state={state} setState={setState} />}
      </DialogContent>
    </Dialog>
  );
};
```

---

### STEP 1: Select Content

**UI Elements:**
```
┌─────────────────────────────────────────────────────────┐
│ Select Website Content                                  │
│                                                          │
│ ┌─────────────────────────────────────────────────┐    │
│ │   Drag folder here or click to browse           │    │
│ │        [ Choose Folder ]                         │    │
│ └─────────────────────────────────────────────────┘    │
│                                                          │
│ 📁 /Users/alice/my-site                                 │
│                                                          │
│ Scan Results:                                           │
│ Files: 124  •  Size: 18.3 MB  •  Blocks: 48             │
│                                                          │
│ Entry File: [index.html        ▼]                       │
│                                                          │
│ Exclude Patterns:                                       │
│ [.git] [node_modules] [.*] [*.tmp] [+ Add]              │
│                                                          │
│ Preview:                                                │
│ ✓ index.html (12.4 KB)                                  │
│ ✓ style.css (3.2 KB)                                    │
│ ✓ script.js (8.1 KB)                                    │
│ ✓ images/logo.png (142 KB)                              │
│ ... 120 more files                                      │
│                                                          │
│                               [Cancel]  [Next →]        │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**

```typescript
const Step1_SelectContent: React.FC<StepProps> = ({ state, setState }) => {
  const handleFolderSelect = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    
    if (selected) {
      // Scan folder
      const scanResult = await invoke<ScanResult>('sites_scan_folder', {
        path: selected,
        excludePatterns: state.excludePatterns,
      });
      
      setState(s => ({
        ...s,
        selectedPath: selected,
        scannedFiles: scanResult.files,
        totalBytes: scanResult.totalBytes,
        estimatedBlocks: scanResult.estimatedBlocks,
      }));
    }
  };
  
  const handleNext = () => {
    if (!state.selectedPath || state.scannedFiles.length === 0) {
      // Show error
      return;
    }
    setState(s => ({ ...s, step: 2 }));
  };
  
  return (
    <Box>
      <FolderPicker onSelect={handleFolderSelect} />
      {state.selectedPath && (
        <>
          <FileScanPreview 
            files={state.scannedFiles}
            totalBytes={state.totalBytes}
            estimatedBlocks={state.estimatedBlocks}
          />
          <TextField
            select
            label="Entry File"
            value={state.entryFile}
            onChange={(e) => setState(s => ({ ...s, entryFile: e.target.value }))}
          >
            {state.scannedFiles
              .filter(f => f.name.endsWith('.html'))
              .map(f => <MenuItem value={f.path}>{f.name}</MenuItem>)}
          </TextField>
        </>
      )}
      <Button onClick={handleNext} disabled={!state.selectedPath}>Next</Button>
    </Box>
  );
};
```

**Backend IPC Needed:**
```rust
#[tauri::command]
async fn sites_scan_folder(
    path: String,
    exclude_patterns: Vec<String>,
) -> Result<ScanResult, String> {
    // Walk directory
    // Apply exclude patterns
    // Calculate size
    // Estimate blocks (totalBytes / MAX_BLOCK_SIZE)
    // Return { files, totalBytes, estimatedBlocks }
}
```

---

### STEP 2: Name & Key

**UI Elements:**
```
┌─────────────────────────────────────────────────────────┐
│ Claim Four-Word Name                                    │
│                                                          │
│ Four-Words:                                             │
│ [ocean-forest-moon-star          ] [Generate Random]    │
│ ✓ Valid dictionary words                                │
│                                                          │
│ [ Check Availability ]                                  │
│                                                          │
│ Status: Available ✓                                     │
│                                                          │
│ Site Key (ML-DSA-87):                                   │
│ ● Create new key                                        │
│ ○ Import existing key                                   │
│                                                          │
│ Key Fingerprint: 8F2C-3A7B-D4E1-...                     │
│ [Copy] [Export Recovery Kit]                            │
│                                                          │
│ ⚠️ Warning: Back up your recovery kit before publishing │
│                                                          │
│                         [← Back]  [Next →]              │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**

```typescript
const Step2_NameAndKey: React.FC<StepProps> = ({ state, setState }) => {
  const [checking, setChecking] = useState(false);
  const [available, setAvailable] = useState<boolean | null>(null);
  
  const handleGenerateRandom = async () => {
    const words = await invoke<string>('identity_generate_four_words');
    setState(s => ({ ...s, fourWordsName: words }));
  };
  
  const handleCheckAvailability = async () => {
    setChecking(true);
    try {
      const result = await invoke<NameCheckResult>('names_check_availability', {
        fourWords: state.fourWordsName,
      });
      
      setAvailable(result.available);
      
      if (!result.available) {
        setState(s => ({ 
          ...s, 
          nameConflict: {
            existingSiteId: result.existingSiteId,
            firstSeen: result.firstSeen,
          }
        }));
      }
    } finally {
      setChecking(false);
    }
  };
  
  const handleGenerateKey = async () => {
    const keypair = await invoke<KeyPair>('sites_generate_keypair');
    
    setState(s => ({
      ...s,
      publicKey: new Uint8Array(keypair.publicKey),
      privateKey: new Uint8Array(keypair.privateKey),
      keyFingerprint: keypair.fingerprint,
    }));
  };
  
  useEffect(() => {
    if (state.siteKeyType === 'generate' && !state.publicKey) {
      handleGenerateKey();
    }
  }, [state.siteKeyType]);
  
  return (
    <Box>
      <FourWordsInput
        value={state.fourWordsName}
        onChange={(v) => setState(s => ({ ...s, fourWordsName: v }))}
        onGenerate={handleGenerateRandom}
        error={!state.nameValid}
      />
      
      <Button onClick={handleCheckAvailability} disabled={!state.nameValid || checking}>
        {checking ? <CircularProgress size={20} /> : 'Check Availability'}
      </Button>
      
      {available === false && (
        <Alert severity="warning">
          Name "{state.fourWordsName}" is already claimed (TOFU).
          Choose a different name or contact the current owner.
        </Alert>
      )}
      
      {available === true && (
        <Alert severity="success">
          "{state.fourWordsName}" is available! ✓
        </Alert>
      )}
      
      <SiteKeyManager 
        keyType={state.siteKeyType}
        onKeyTypeChange={(type) => setState(s => ({ ...s, siteKeyType: type }))}
        fingerprint={state.keyFingerprint}
        onExportRecoveryKit={handleExportRecoveryKit}
      />
    </Box>
  );
};
```

**Backend IPC Needed:**
```rust
#[tauri::command]
async fn identity_generate_four_words() -> Result<String, String> {
    Ok(communitas_core::identity::generate_id_words()?)
}

#[tauri::command]
async fn names_check_availability(four_words: String) -> Result<NameCheckResult, String> {
    // Check NameRegistry.resolve(four_words)
    // If exists, return { available: false, existingSiteId }
    // Else return { available: true }
}

#[tauri::command]
async fn sites_generate_keypair() -> Result<KeyPair, String> {
    // Generate ML-DSA-87 keypair
    // Return { publicKey: Vec<u8>, privateKey: Vec<u8>, fingerprint: String }
}
```

---

### STEP 3: Sign & Review

**UI Elements:**
```
┌─────────────────────────────────────────────────────────┐
│ Sign & Review                                           │
│                                                          │
│ Manifest Details:                                       │
│ Version: [2        ] (auto-increment)                   │
│ Timestamp: 2025-10-29 14:23:41                          │
│                                                          │
│ Content Summary:                                        │
│ Files: 124  •  Blocks: 48  •  Total: 18.3 MB            │
│                                                          │
│ ┌─ Preview (first 5 files) ─────────────────────┐      │
│ │ Path              Size      BLAKE3 Hash       │      │
│ │ index.html        12.4 KB   8f2c3a7b...      │      │
│ │ style.css          3.2 KB   a1b2c3d4...      │      │
│ │ script.js          8.1 KB   e5f6g7h8...      │      │
│ │ images/logo.png  142.0 KB   9i0j1k2l...      │      │
│ │ ... 120 more files                            │      │
│ └───────────────────────────────────────────────┘      │
│                                                          │
│ Actions:                                                │
│ [ Sign Manifest ] [ Sign Name Record ]                  │
│                                                          │
│ Security Status:                                        │
│ 🔒 Manifest: Signed (ML-DSA-87 • FP: 8F2C...)          │
│ 🔒 Name Record: Signed ("ocean-forest-moon-star")      │
│                                                          │
│                         [← Back]  [Publish →]           │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**

```typescript
const Step3_SignReview: React.FC<StepProps> = ({ state, setState }) => {
  const handleSignManifest = async () => {
    try {
      // Build manifest
      const manifest = await invoke<SiteManifest>('sites_build_manifest', {
        siteId: state.siteId, // Derived from publicKey
        publicKey: Array.from(state.publicKey!),
        manifestVersion: state.manifestVersion,
        blockMap: state.blockMap, // Built from scanned files
      });
      
      // Sign manifest
      const signedManifest = await invoke<SiteManifest>('sites_sign_manifest', {
        manifest,
        privateKey: Array.from(state.privateKey!),
      });
      
      setState(s => ({
        ...s,
        manifest: signedManifest,
        manifestSigned: true,
      }));
    } catch (err) {
      setError(`Signing failed: ${err}`);
    }
  };
  
  const handleSignNameRecord = async () => {
    try {
      const record = await invoke<NameRecord>('names_create_and_sign', {
        fourWords: state.fourWordsName,
        publicKey: Array.from(state.publicKey!),
        privateKey: Array.from(state.privateKey!),
      });
      
      setState(s => ({
        ...s,
        nameRecord: record,
        nameRecordSigned: true,
      }));
    } catch (err) {
      setError(`Name record signing failed: ${err}`);
    }
  };
  
  return (
    <Box>
      <ManifestPreview 
        files={state.scannedFiles}
        blocks={state.estimatedBlocks}
        totalSize={state.totalBytes}
      />
      
      <TextField
        label="Manifest Version"
        type="number"
        value={state.manifestVersion}
        onChange={(e) => setState(s => ({ ...s, manifestVersion: parseInt(e.target.value) }))}
      />
      
      <Stack direction="row" spacing={2}>
        <Button 
          variant="contained" 
          onClick={handleSignManifest}
          disabled={state.manifestSigned}
        >
          {state.manifestSigned ? '✓ Manifest Signed' : 'Sign Manifest'}
        </Button>
        
        <Button 
          variant="contained" 
          onClick={handleSignNameRecord}
          disabled={state.nameRecordSigned}
        >
          {state.nameRecordSigned ? '✓ Name Record Signed' : 'Sign Name Record'}
        </Button>
      </Stack>
      
      {state.manifestSigned && state.nameRecordSigned && (
        <PQCLockIndicator 
          status="verified" 
          fingerprint={state.keyFingerprint}
          algorithm="ML-DSA-87"
        />
      )}
    </Box>
  );
};
```

---

### STEP 4: Publish & Seed

**UI Elements:**
```
┌─────────────────────────────────────────────────────────┐
│ Publishing...                                           │
│                                                          │
│ Timeline:                                               │
│ ✓ Chunked    ✓ Cached    ✓ Signed    ↻ Announcing      │
│                                                          │
│ Blocks Prepared: [████████████░░░░] 32/48               │
│                                                          │
│ Provider Status:                                        │
│ ✓ Listening on QUIC                                     │
│ ✓ Announced to rendezvous shard                         │
│ ℹ️ Seeding from this device                             │
│                                                          │
│ Share Your Site:                                        │
│ ┌───────────────────────────────────────┐              │
│ │ ocean-forest-moon-star     [Copy]     │              │
│ │ SiteId: 8f2c3a7b...        [Copy]     │              │
│ │ [Open in Viewer]                      │              │
│ └───────────────────────────────────────┘              │
│                                                          │
│                                  [View Logs]  [Done]    │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**

```typescript
const Step4_PublishSeed: React.FC<StepProps> = ({ state, setState }) => {
  useEffect(() => {
    // Start publication process
    const publish = async () => {
      try {
        await invoke('sites_start_provider', {
          siteId: state.siteId,
          manifest: state.manifest,
          nameRecord: state.nameRecord,
        });
        
        setState(s => ({ ...s, publishPhase: 'done' }));
      } catch (err) {
        setState(s => ({ 
          ...s, 
          publishPhase: 'error',
          errors: [...s.errors, { message: err.toString() }],
        }));
      }
    };
    
    publish();
  }, []);
  
  return (
    <Box>
      <PublishProgress 
        phase={state.publishPhase}
        blocksPrepared={state.blocksPrepared}
        blocksTotal={state.blocksTotal}
      />
      
      {state.publishPhase === 'done' && (
        <ShareCard 
          fourWords={state.fourWordsName}
          siteId={state.siteId}
          onOpenViewer={() => {/* Open viewer with this site */}}
        />
      )}
      
      {state.publishPhase === 'error' && (
        <Alert severity="error">
          {state.errors.map(e => <div key={e.message}>{e.message}</div>)}
          <Button onClick={handleRetry}>Retry</Button>
        </Alert>
      )}
    </Box>
  );
};
```

---

## 🌐 VIEWER COMPONENT

### Main Viewer Layout

**UI Elements:**
```
┌─────────────────────────────────────────────────────────┐
│ [←] [→] [⟳]  [ocean-forest-moon-star    ] [★] [ℹ️]     │
│                                                          │
│ 🔒 PQC Verified  •  Providers: 3 (LAN:2, Remote:1)     │
│ Fetching blocks: [████████░░░] 32/48  (12.4/18.3 MB)   │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                                                          │
│        [ Rendered Website Content ]                     │
│          (Sandboxed WebView)                            │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

**Implementation:**

```typescript
interface ViewerState {
  fourWords: string;
  siteId: SiteId | null;
  
  // Discovery
  resolving: boolean;
  providers: Provider[];
  
  // Fetch
  fetchingManifest: boolean;
  manifest: SiteManifest | null;
  manifestVerified: boolean;
  
  fetchingBlocks: boolean;
  blocksFetched: number;
  blocksTotal: number;
  bytesFetched: number;
  bytesTotal: number;
  
  // Render
  rendering: boolean;
  renderedHtml: string | null;
  
  // Security
  tofuStatus: 'new' | 'known' | 'mismatch';
  keyFingerprint: string;
  pinned: boolean;
  
  // Cache
  offline: boolean;
  cachedManifest: SiteManifest | null;
  
  // Error
  error: string | null;
}

const Viewer: React.FC = () => {
  const [state, setState] = useState<ViewerState>({
    fourWords: '',
    siteId: null,
    providers: [],
    // ... defaults
  });
  
  const [history, setHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [bookmarks, setBookmarks] = useState<string[]>([]);
  const [drawerOpen, setDrawerOpen] = useState(false);
  
  // Navigation
  const handleNavigate = async (fourWords: string) => {
    // Add to history
    setHistory(h => [...h, fourWords]);
    setHistoryIndex(history.length);
    
    setState(s => ({
      ...s,
      fourWords,
      resolving: true,
      error: null,
    }));
    
    try {
      // Step 1: Resolve name
      const siteId = await invoke<string>('names_resolve', { fourWords });
      
      if (!siteId) {
        setState(s => ({ ...s, error: 'Name not found', resolving: false }));
        return;
      }
      
      setState(s => ({ ...s, siteId, resolving: false }));
      
      // Step 2: Discover providers
      await discoverProviders(siteId);
      
      // Step 3: Fetch manifest
      await fetchManifest(siteId);
      
      // Step 4: Fetch blocks
      await fetchBlocks(siteId);
      
      // Step 5: Render
      await renderSite();
      
    } catch (err) {
      setState(s => ({ ...s, error: err.toString(), resolving: false }));
    }
  };
  
  const discoverProviders = async (siteId: string) => {
    setState(s => ({ ...s, discovering: true }));
    
    // Subscribe to discovery events
    const unlisten = await listen<Provider[]>('sites:discovery', (event) => {
      setState(s => ({ ...s, providers: event.payload }));
    });
    
    // Start discovery
    await invoke('sites_subscribe_discovery', { siteId });
    
    // Wait up to 5 seconds for providers
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    unlisten();
  };
  
  const fetchManifest = async (siteId: string) => {
    setState(s => ({ ...s, fetchingManifest: true }));
    
    const manifest = await invoke<SiteManifest>('sites_fetch_manifest', {
      siteId,
      providerId: state.providers[0]?.id, // Best provider
    });
    
    // Verify signature
    const verified = await invoke<boolean>('sites_verify_manifest', { manifest });
    
    if (!verified) {
      throw new Error('Manifest signature verification failed');
    }
    
    // TOFU check
    const tofuStatus = await invoke<TOFUStatus>('security_check_tofu', {
      siteId,
      publicKey: manifest.publicKey,
    });
    
    setState(s => ({
      ...s,
      manifest,
      manifestVerified: true,
      tofuStatus: tofuStatus.status,
      keyFingerprint: tofuStatus.fingerprint,
      fetchingManifest: false,
    }));
    
    // Show TOFU dialog if first time
    if (tofuStatus.status === 'new') {
      const shouldTrust = await showTOFUDialog(tofuStatus);
      if (!shouldTrust) {
        throw new Error('User rejected key');
      }
    }
  };
  
  const fetchBlocks = async (siteId: string) => {
    setState(s => ({ 
      ...s, 
      fetchingBlocks: true,
      blocksTotal: s.manifest!.blocks.length,
    }));
    
    // Listen to fetch progress
    const unlisten = await listen<FetchProgress>('sites:fetch_progress', (event) => {
      setState(s => ({
        ...s,
        blocksFetched: event.payload.blocksFetched,
        bytesFetched: event.payload.bytesFetched,
      }));
    });
    
    // Fetch all blocks (backend handles concurrency)
    await invoke('sites_fetch_blocks', {
      siteId,
      manifest: state.manifest,
      concurrency: 6, // Parallel fetches
    });
    
    unlisten();
    setState(s => ({ ...s, fetchingBlocks: false }));
  };
  
  return (
    <Box sx={{ height: '100vh', display: 'flex', flexDirection: 'column' }}>
      <ViewerToolbar
        fourWords={state.fourWords}
        onNavigate={handleNavigate}
        onBack={() => history[historyIndex - 1] && handleNavigate(history[historyIndex - 1])}
        onForward={() => history[historyIndex + 1] && handleNavigate(history[historyIndex + 1])}
        onBookmark={() => setBookmarks(b => [...b, state.fourWords])}
        onInfo={() => setDrawerOpen(true)}
        pqcStatus={state.manifestVerified ? 'verified' : 'unknown'}
        offline={state.offline}
      />
      
      {(state.resolving || state.fetchingManifest || state.fetchingBlocks) && (
        <DiscoveryStatusBar
          resolving={state.resolving}
          providers={state.providers}
          fetchingManifest={state.fetchingManifest}
          manifestVerified={state.manifestVerified}
          fetchProgress={{
            blocksFetched: state.blocksFetched,
            blocksTotal: state.blocksTotal,
            bytesFetched: state.bytesFetched,
            bytesTotal: state.bytesTotal,
          }}
        />
      )}
      
      <ContentPanel 
        html={state.renderedHtml}
        loading={state.rendering}
      />
      
      <SiteInfoDrawer
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        site={{
          fourWords: state.fourWords,
          siteId: state.siteId,
          manifest: state.manifest,
        }}
        security={{
          status: state.tofuStatus,
          fingerprint: state.keyFingerprint,
          pinned: state.pinned,
        }}
        providers={state.providers}
        cache={state.cachedManifest}
      />
    </Box>
  );
};
```

---

## 🔒 SECURITY & TOFU COMPONENTS

### PQCLockIndicator

```typescript
interface PQCLockIndicatorProps {
  status: 'verified' | 'tofu' | 'mismatch' | 'unknown' | 'loading';
  fingerprint?: string;
  algorithm?: string; // "ML-DSA-87"
  onClick?: () => void;
}

const PQCLockIndicator: React.FC<PQCLockIndicatorProps> = ({
  status,
  fingerprint,
  algorithm = 'ML-DSA-87',
  onClick,
}) => {
  const getIcon = () => {
    switch (status) {
      case 'verified': return <LockIcon sx={{ color: 'success.main' }} />;
      case 'tofu': return <InfoIcon sx={{ color: 'warning.main' }} />;
      case 'mismatch': return <WarningIcon sx={{ color: 'error.main' }} />;
      case 'loading': return <CircularProgress size={16} />;
      default: return <LockOpenIcon sx={{ color: 'text.disabled' }} />;
    }
  };
  
  const getLabel = () => {
    switch (status) {
      case 'verified': return 'PQC Verified';
      case 'tofu': return 'First Seen (TOFU)';
      case 'mismatch': return 'KEY CHANGED!';
      default: return 'Not Verified';
    }
  };
  
  return (
    <Tooltip title={`${algorithm} • ${fingerprint || 'No key'}`}>
      <Chip
        icon={getIcon()}
        label={getLabel()}
        onClick={onClick}
        size="small"
        sx={{ cursor: onClick ? 'pointer' : 'default' }}
      />
    </Tooltip>
  );
};
```

### TOFU Dialog

```typescript
const TOFUDialog: React.FC<TOFUDialogProps> = ({
  open,
  fourWords,
  siteId,
  fingerprint,
  createdAt,
  onTrust,
  onReject,
}) => {
  return (
    <Dialog open={open}>
      <DialogTitle>First Time Seeing This Site</DialogTitle>
      <DialogContent>
        <Alert severity="info" sx={{ mb: 2 }}>
          This site's post-quantum key was first seen now.
          Trust and pin to remember it for future visits.
        </Alert>
        
        <List dense>
          <ListItem>
            <ListItemText primary="Name" secondary={fourWords} />
          </ListItem>
          <ListItem>
            <ListItemText primary="Site ID" secondary={siteId} />
          </ListItem>
          <ListItem>
            <ListItemText 
              primary="Key Fingerprint (ML-DSA-87)" 
              secondary={fingerprint}
            />
            <IconButton size="small" onClick={() => copyToClipboard(fingerprint)}>
              <CopyIcon />
            </IconButton>
          </ListItem>
          <ListItem>
            <ListItemText primary="First Seen" secondary={new Date(createdAt).toLocaleString()} />
          </ListItem>
        </List>
        
        <Typography variant="caption" color="text.secondary">
          Verify this fingerprint through a trusted channel before trusting.
        </Typography>
      </DialogContent>
      <DialogActions>
        <Button onClick={onReject} color="error">
          Reject
        </Button>
        <Button onClick={() => onTrust(false)}>
          Trust Once
        </Button>
        <Button onClick={() => onTrust(true)} variant="contained" autoFocus>
          Trust & Pin
        </Button>
      </DialogActions>
    </Dialog>
  );
};
```

---

## 📊 PROGRESS INDICATORS

### FetchProgress Component

```typescript
const FetchProgress: React.FC<FetchProgressProps> = ({
  blocksFetched,
  blocksTotal,
  bytesFetched,
  bytesTotal,
  speedBps,
  etaSec,
}) => {
  const progress = (blocksFetched / blocksTotal) * 100;
  const speedMbps = speedBps ? (speedBps / 1024 / 1024).toFixed(1) : null;
  
  return (
    <Box>
      <Stack direction="row" spacing={2} alignItems="center">
        <Box sx={{ flex: 1 }}>
          <LinearProgress variant="determinate" value={progress} />
        </Box>
        <Typography variant="caption">
          {blocksFetched}/{blocksTotal} blocks
        </Typography>
      </Stack>
      
      <Stack direction="row" spacing={2} sx={{ mt: 0.5 }}>
        <Typography variant="caption" color="text.secondary">
          {formatBytes(bytesFetched)} / {formatBytes(bytesTotal)}
        </Typography>
        {speedMbps && (
          <Typography variant="caption" color="text.secondary">
            {speedMbps} MB/s
          </Typography>
        )}
        {etaSec && (
          <Typography variant="caption" color="text.secondary">
            ETA: {formatDuration(etaSec)}
          </Typography>
        )}
      </Stack>
    </Box>
  );
};
```

### ProviderList Component

```typescript
const ProviderList: React.FC<ProviderListProps> = ({
  providers,
  onPrefer,
  onBan,
}) => {
  return (
    <List>
      {providers.map(provider => (
        <ListItem key={provider.id}>
          <ListItemIcon>
            {provider.type === 'lan' ? <WifiIcon /> : <CloudIcon />}
          </ListItemIcon>
          <ListItemText
            primary={provider.fourWordsAddr || 'Unknown'}
            secondary={
              <Stack direction="row" spacing={1} alignItems="center">
                <Chip 
                  label={provider.state} 
                  size="small"
                  color={provider.state === 'active' ? 'success' : 'default'}
                />
                {provider.latencyMs && (
                  <Typography variant="caption">{provider.latencyMs}ms</Typography>
                )}
                <Typography variant="caption" color="text.secondary">
                  Score: {provider.score}/100
                </Typography>
              </Stack>
            }
          />
          <ListItemSecondaryAction>
            <IconButton onClick={() => onPrefer(provider.id)} size="small">
              <StarIcon />
            </IconButton>
            <IconButton onClick={() => onBan(provider.id)} size="small">
              <BlockIcon />
            </IconButton>
          </ListItemSecondaryAction>
        </ListItem>
      ))}
    </List>
  );
};
```

---

## 📱 COMPLETE BACKEND IPC SURFACE

### Publisher Commands

```rust
// Step 1: Scan
#[tauri::command]
async fn sites_scan_folder(path: String, exclude_patterns: Vec<String>) -> Result<ScanResult, String>

// Step 2: Name & Key
#[tauri::command]
async fn identity_generate_four_words() -> Result<String, String>

#[tauri::command]
async fn names_check_availability(four_words: String) -> Result<NameCheckResult, String>

#[tauri::command]
async fn sites_generate_keypair() -> Result<KeyPair, String>

#[tauri::command]
async fn sites_export_recovery_kit(private_key: Vec<u8>, password: String) -> Result<Vec<u8>, String>

// Step 3: Sign
#[tauri::command]
async fn sites_build_manifest(
    site_id: String,
    public_key: Vec<u8>,
    manifest_version: u64,
    files: Vec<FileInfo>,
) -> Result<SiteManifest, String>

#[tauri::command]
async fn sites_sign_manifest(
    manifest: SiteManifest,
    private_key: Vec<u8>,
) -> Result<SiteManifest, String>

#[tauri::command]
async fn names_create_and_sign(
    four_words: String,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
) -> Result<NameRecord, String>

// Step 4: Publish
#[tauri::command]
async fn sites_start_provider(
    site_id: String,
    manifest: SiteManifest,
    name_record: Option<NameRecord>,
) -> Result<(), String>

#[tauri::command]
async fn sites_stop_provider(site_id: String) -> Result<(), String>
```

### Viewer Commands

```rust
// Discovery
#[tauri::command]
async fn names_resolve(four_words: String) -> Result<Option<String>, String>

#[tauri::command]
async fn sites_subscribe_discovery(site_id: String) -> Result<(), String>
// Emits: sites:discovery { providers: Provider[] }

// Fetching
#[tauri::command]
async fn sites_fetch_manifest(site_id: String, provider_id: Option<String>) -> Result<SiteManifest, String>

#[tauri::command]
async fn sites_verify_manifest(manifest: SiteManifest) -> Result<bool, String>

#[tauri::command]
async fn sites_fetch_blocks(
    site_id: String,
    manifest: SiteManifest,
    concurrency: usize,
) -> Result<(), String>
// Emits: sites:fetch_progress { blocksFetched, bytesFetched, ... }

// Security
#[tauri::command]
async fn security_check_tofu(
    site_id: String,
    public_key: Vec<u8>,
) -> Result<TOFUStatus, String>

#[tauri::command]
async fn security_trust_and_pin(site_id: String, public_key: Vec<u8>) -> Result<(), String>

// Cache
#[tauri::command]
async fn sites_get_cached(site_id: String) -> Result<Option<CachedSite>, String>

#[tauri::command]
async fn sites_pin_cache(site_id: String) -> Result<(), String>

#[tauri::command]
async fn sites_clear_cache(site_id: String) -> Result<(), String>
```

---

## 🎨 USER FLOWS

### Flow 1: First-Time Publisher

```
User Action                Backend                     UI Feedback
───────────────────────────────────────────────────────────────────
1. Click "Publish Website"  
                           Create wizard state         Show Step 1

2. Choose folder           scan_folder()               "Scanning... 124 files"
                           Build file list             Show preview

3. Click Next              
                           Validate                    Move to Step 2

4. Click "Generate Random" generate_four_words()       "ocean-forest-moon-star"

5. Click "Check"           names_check_availability()  "Available ✓"

6. Auto-generate key       sites_generate_keypair()    Show fingerprint

7. Click "Export Kit"      sites_export_recovery_kit() Download .kit file
                           
8. Click Next                                          Move to Step 3

9. Review manifest                                     Show preview table

10. Click "Sign Manifest"  sites_sign_manifest()       🔒 Signed (green)

11. Click "Sign Name"      names_create_and_sign()     🔒 Signed (green)

12. Click "Publish"        sites_start_provider()      Move to Step 4
                           Chunking...                 Progress: "Chunking"
                           Caching blocks              Progress: "Cached 32/48"
                           Announce to rendezvous      Progress: "Announcing"
                           Start provider              Progress: "Seeding"
                           
13. Publication done                                   Show share card
                                                       "Copy four-words"
                                                       "Open in Viewer"
```

### Flow 2: First-Time Viewer

```
User Action                Backend                     UI Feedback
───────────────────────────────────────────────────────────────────
1. Type four-words         
   "ocean-forest-moon-star"

2. Press Enter             names_resolve()             "Resolving..."
                           Found SiteId                "Resolved ✓"
                           
3. Auto-discover           sites_subscribe_discovery() "Finding providers..."
                           Found 3 providers           "Providers: 3 (LAN:2)"

4. Fetch manifest          sites_fetch_manifest()      "Fetching manifest..."
                           Verify signature            "Verifying signature..."
                           Check TOFU                  Show TOFU dialog
                           
5. User clicks "Trust&Pin" security_trust_and_pin()     Close dialog
                                                       🔒 PQC Verified (green)

6. Fetch blocks            sites_fetch_blocks()        Progress bar
                           Block 1/48...               "Fetching 1/48"
                           Block 48/48                 "Fetching 48/48"
                           
7. Render content          Assemble HTML from blocks   Show website!
                           Sandbox restrictions        
                           
8. Browse content                                      Normal website UX
```

### Flow 3: Offline Viewing

```
User Action                Backend                     UI Feedback
───────────────────────────────────────────────────────────────────
1. Type known four-words   names_resolve()             "Resolving..."
                           Found in cache              "Resolved ✓"

2. Discover providers      sites_subscribe_discovery() "Finding providers..."
                           No providers found          "No providers"
                           Check cache                 "Offline mode ⚡"
                           
3. Load from cache         sites_get_cached()          "Loading from cache..."
                           Load manifest               Show version/date
                           Load blocks                 Show content
                           
4. View content                                        Show with offline badge
                                                       "Offline (from cache)"
                                                       "Manifest v2 • Jan 28"
```

### Flow 4: Update Published Site

```
User Action                Backend                     UI Feedback
───────────────────────────────────────────────────────────────────
1. Open "Manage Sites"     sites_list_published()      Show sites list

2. Click "Update" on site  Load previous state         Pre-fill wizard
                           folder, name, key           

3. Modify files            Rescan folder               Show diff:
   (add/edit/delete)                                   +3 added
                                                       ~2 modified
                                                       -1 deleted

4. Click through wizard    Auto-increment version      v2 → v3
                           to Step 3                   

5. Sign new manifest       sites_sign_manifest()       🔒 v3 Signed
                           manifestVersion: 3          

6. Publish                 sites_start_provider()      Update provider
                           Stop v2 provider            Show "Updated to v3"
                           Start v3 provider           
```

---

## 🚨 ERROR STATES & HANDLING

### Critical Error States

**1. Name Conflict (TOFU)**
```
┌─────────────────────────────────────────────┐
│ ⚠️ Name Already Claimed                     │
│                                              │
│ "ocean-forest-moon-star" is already bound   │
│ to a different site key (TOFU principle).   │
│                                              │
│ First seen: Jan 15, 2025                    │
│ Owner's SiteId: 8f2c3a7b...                 │
│                                              │
│ Options:                                    │
│ • Choose a different four-words name        │
│ • Contact the current owner                 │
│ • Use your SiteId directly (advanced)       │
│                                              │
│      [Choose Different Name]  [Cancel]      │
└─────────────────────────────────────────────┘
```

**2. No Providers Found**
```
┌─────────────────────────────────────────────┐
│ 🔍 No Providers Found                       │
│                                              │
│ "cool-site" resolved to SiteId 8f2c...      │
│ but no providers are currently seeding it.  │
│                                              │
│ Checked:                                    │
│ • Rendezvous shards (0 providers)           │
│ • LAN broadcast (none found)                │
│ • Bootstrap nodes (unreachable)             │
│                                              │
│ Possible reasons:                           │
│ • Publisher is offline                      │
│ • Network partition                         │
│ • Site no longer exists                     │
│                                              │
│ [Retry] [Use Cached Version] [Cancel]       │
└─────────────────────────────────────────────┘
```

**3. Signature Mismatch (Security Alert)**
```
┌─────────────────────────────────────────────┐
│ 🚨 SECURITY WARNING: Key Changed            │
│                                              │
│ The site "ocean-forest-moon-star" is now    │
│ using a DIFFERENT ML-DSA key than before.   │
│                                              │
│ Previous key (pinned):                      │
│ FP: 8f2c-3a7b-d4e1-...                      │
│ First seen: Jan 15, 2025                    │
│                                              │
│ New key:                                    │
│ FP: a1b2-c3d4-e5f6-...                      │
│ First seen: Jan 29, 2025                    │
│                                              │
│ ⚠️ This could be:                            │
│ • Legitimate key rotation by owner          │
│ • Man-in-the-middle attack                  │
│                                              │
│ Verify with publisher through trusted       │
│ channel before proceeding!                  │
│                                              │
│ [Reject & Block]  [View Details]  [Trust]   │
└─────────────────────────────────────────────┘
```

**4. Fetch Timeout/Failure**
```
┌─────────────────────────────────────────────┐
│ ⏱️ Fetch Timed Out                          │
│                                              │
│ Failed to fetch blocks 12-18 from provider  │
│ ocean-blue-eagle-star (192.168.1.42).       │
│                                              │
│ Progress: 11/48 blocks (2.3 MB / 18.3 MB)   │
│                                              │
│ Automatically trying next provider:         │
│ forest-river-mountain-cloud (remote)        │
│                                              │
│ [ View Provider Details ]                   │
└─────────────────────────────────────────────┘
```

---

## 💾 OFFLINE-FIRST EXPERIENCE

### Offline Indicators

**OfflineChip Component:**
```typescript
const OfflineChip: React.FC<{ 
  cached: boolean;
  manifestVersion?: number;
  cachedDate?: Date;
}> = ({ cached, manifestVersion, cachedDate }) => {
  if (!cached) return null;
  
  return (
    <Chip
      icon={<CloudOffIcon />}
      label={`Offline (v${manifestVersion} • ${formatDate(cachedDate)})`}
      size="small"
      color="warning"
      variant="outlined"
    />
  );
};
```

**Cache Management in Site Info Drawer:**
```
┌─ Cache ───────────────────────────────────────┐
│ Status: Fully cached (18.3 MB)                 │
│                                                │
│ Manifest: v2                                   │
│ Cached: Jan 28, 2025 14:23                     │
│ Blocks: 48/48 (100%)                           │
│                                                │
│ [ Pin for Offline ] (currently pinned)         │
│ [ Clear Cache ]                                │
│                                                │
│ Offline viewing: Available ✓                   │
└────────────────────────────────────────────────┘
```

---

## 🎓 ONBOARDING

### First-Launch Welcome

```typescript
const OnboardingDialog: React.FC = () => {
  return (
    <Dialog open maxWidth="md">
      <DialogTitle>Welcome to Communitas Websites</DialogTitle>
      <DialogContent>
        <Typography variant="h6" gutterBottom>
          This is NOT the traditional web
        </Typography>
        
        <Stack spacing={3} sx={{ my: 3 }}>
          <Box>
            <Typography variant="subtitle1" gutterBottom>
              📛 Type four-words, not URLs
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Instead of "www.example.com", use "ocean-forest-moon-star"
            </Typography>
          </Box>
          
          <Box>
            <Typography variant="subtitle1" gutterBottom>
              🌐 Mesh network discovery
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Communitas finds providers via peer-to-peer rendezvous,
              not DNS servers
            </Typography>
          </Box>
          
          <Box>
            <Typography variant="subtitle1" gutterBottom>
              🔒 Post-quantum signatures
            </Typography>
            <Typography variant="body2" color="text.secondary">
              All sites are signed with ML-DSA-87, not HTTPS certificates
            </Typography>
          </Box>
        </Stack>
        
        <FormControlLabel
          control={<Checkbox defaultChecked />}
          label="Show advanced network details"
        />
      </DialogContent>
      <DialogActions>
        <Button href="/docs/four-words-guide">Learn More</Button>
        <Button variant="contained" onClick={handleClose}>Get Started</Button>
      </DialogActions>
    </Dialog>
  );
};
```

---

## ⚙️ SETTINGS & PREFERENCES

### Websites Settings Panel

**Location:** Settings → Websites

```
┌─ Publishing ──────────────────────────────────┐
│ Default exclude patterns:                      │
│ [.git, node_modules, .*, *.tmp]               │
│                                                │
│ Default entry file: [index.html      ▼]       │
│                                                │
│ Auto-increment manifest version: [✓]           │
│                                                │
│ Require recovery kit export: [✓]              │
└────────────────────────────────────────────────┘

┌─ Viewing ─────────────────────────────────────┐
│ Prefer LAN providers: [✓]                     │
│                                                │
│ Max parallel block fetches: [6        ]        │
│                                                │
│ Auto-pin viewed sites: [  ]                    │
│                                                │
│ Show raw discovery logs: [  ]                  │
└────────────────────────────────────────────────┘

┌─ Security ────────────────────────────────────┐
│ Require confirmation on key change: [✓]       │
│                                                │
│ Show raw SiteId hashes: [✓]                   │
│                                                │
│ [Export Trust Store] [Import Trust Store]     │
└────────────────────────────────────────────────┘

┌─ Cache ───────────────────────────────────────┐
│ Max cache size: [1 GB      ▼]                 │
│                                                │
│ Default TTL: [7 days    ▼]                    │
│                                                │
│ Current usage: 234 MB / 1 GB (23%)             │
│                                                │
│ [Clear All Unpinned] [Clear All]               │
└────────────────────────────────────────────────┘
```

---

## 📐 IMPLEMENTATION ROADMAP

### Day 1: Shared Components & Publisher Steps 1-2

**Morning (4 hours):**
1. Create `FourWordsInput.tsx` with validation (1h)
2. Create `PQCLockIndicator.tsx` (0.5h)
3. Create `OfflineChip.tsx` (0.5h)
4. Implement Step1_SelectContent (1h)
5. Implement Step2_NameAndKey (1h)

**Afternoon (4 hours):**
6. Backend: `sites_scan_folder` command (1h)
7. Backend: `names_check_availability` command (0.5h)
8. Backend: `sites_generate_keypair` command (0.5h)
9. Test Steps 1-2 end-to-end (1h)
10. Fix bugs, polish UX (1h)

**Deliverable:** Can select content and claim name ✅

---

### Day 2: Publisher Steps 3-4 & Backend Integration

**Morning (4 hours):**
1. Implement Step3_SignReview (1.5h)
2. Implement Step4_PublishSeed (1.5h)
3. Create `PublishProgress.tsx` (1h)

**Afternoon (4 hours):**
4. Backend: `sites_build_manifest` (1h)
5. Backend: `sites_sign_manifest` (0.5h)
6. Backend: `names_create_and_sign` (0.5h)
7. Backend: `sites_start_provider` (1h)
8. Test full publisher flow (1h)

**Deliverable:** Can publish complete site ✅

---

### Day 3: Viewer & Discovery

**Morning (4 hours):**
1. Create `Viewer.tsx` main component (1h)
2. Create `ViewerToolbar.tsx` with address bar (1h)
3. Create `DiscoveryStatusBar.tsx` (1h)
4. Create `ProviderList.tsx` (1h)

**Afternoon (4 hours):**
5. Create `FetchProgress.tsx` (0.5h)
6. Implement `SandboxedWebView.tsx` (1.5h)
7. Backend: `names_resolve` (0.5h)
8. Backend: `sites_subscribe_discovery` + events (1h)
9. Test viewer discovery (0.5h)

**Deliverable:** Can browse to four-words and discover providers ✅

---

### Day 4: Fetching, Security, & Polish

**Morning (4 hours):**
1. Backend: `sites_fetch_manifest` (0.5h)
2. Backend: `sites_fetch_blocks` + progress events (1.5h)
3. Backend: `security_check_tofu` (1h)
4. Implement `TOFUDialog.tsx` (1h)

**Afternoon (4 hours):**
5. Create `SiteInfoDrawer.tsx` (2h)
6. Implement cache management (1h)
7. End-to-end testing (publish → browse → offline) (1h)

**Deliverable:** Complete working system ✅

---

## 🧪 TESTING PLAN

### Unit Tests (per component)

```typescript
describe('FourWordsInput', () => {
  it('validates dictionary words', () => {
    // ocean-forest-moon-star: valid
    // invalid-xyz-bad-words: invalid
  });
  
  it('shows suggestions from history', () => {
    // Type "ocean" → suggests "ocean-forest-moon-star"
  });
});

describe('PublisherWizard', () => {
  it('completes 4-step flow', async () => {
    // Step 1: Select folder
    // Step 2: Claim name
    // Step 3: Sign
    // Step 4: Publish
  });
  
  it('handles name conflicts', async () => {
    // Mock availability check returning conflict
    // Verify error message shown
  });
});

describe('Viewer', () => {
  it('resolves four-words and discovers providers', async () => {
    // Mock names_resolve
    // Mock sites_subscribe_discovery
    // Verify provider list shown
  });
  
  it('handles offline mode gracefully', async () => {
    // Mock no providers
    // Mock cached site available
    // Verify offline badge shown
  });
});
```

### Integration Tests

```typescript
describe('End-to-End Sites Flow', () => {
  it('publishes and browses a site', async () => {
    // 1. Use PublisherWizard to publish
    // 2. Verify NameRecord in registry
    // 3. Use Viewer to browse
    // 4. Verify content matches
    // 5. Verify signatures validate
  });
  
  it('handles update workflow', async () => {
    // 1. Publish site v1
    // 2. Modify content
    // 3. Publish site v2
    // 4. Viewer shows v2
    // 5. Manifest version incremented
  });
});
```

---

## 📦 DEPLOYMENT CHECKLIST

### Before Alpha Launch

- [ ] All 4 wizard steps functional
- [ ] Viewer can browse published sites
- [ ] TOFU dialog works
- [ ] Offline viewing works
- [ ] All error states handled gracefully
- [ ] Recovery kit export works
- [ ] 50+ UI tests passing
- [ ] User documentation written

### Before Beta Launch

- [ ] Multi-provider failover tested
- [ ] Large site performance tested (100+ MB)
- [ ] Key rotation workflow
- [ ] Concurrent publishing from multiple devices
- [ ] Cache eviction tested under pressure
- [ ] Security audit completed

---

## 🎯 SUCCESS CRITERIA

### Minimum Viable UI (Alpha)

**Publisher:**
- User can select a folder ✓
- User can claim a four-word name ✓
- User can sign and publish ✓
- User can see publication status ✓

**Viewer:**
- User can type four-words and navigate ✓
- User can see provider discovery ✓
- User can view content ✓
- User can see security status ✓

**Security:**
- TOFU dialog on first view ✓
- Key mismatch warnings ✓
- Signature verification visible ✓

**Offline:**
- Cache badge when offline ✓
- Can view cached sites ✓
- Can pin important sites ✓

### Complete Feature Set (Beta)

**Publisher:**
- Diff preview for updates
- Multi-device publishing
- Invite additional seeders
- Publishing analytics

**Viewer:**
- Bookmarks
- History
- Full-text search (cached sites)
- Custom CSS injection

---

## 🔧 TECHNICAL NOTES

### WebView Security (Critical!)

The sandboxed WebView MUST enforce:

```typescript
const webviewConfig = {
  csp: {
    defaultSrc: ['none'],
    styleSrc: ['unsafe-inline'], // Allow inline CSS
    imgSrc: ['data:', 'blob:'], // Only local resources
    scriptSrc: ['none'], // NO JavaScript for MVP
    connectSrc: ['none'], // NO network requests
  },
  sandbox: true,
  allowedOrigins: [],
};
```

All resources must be:
- Loaded from BlockCache
- Converted to data: URLs or blob: URLs
- Injected into sandboxed iframe
- NO external network access

### Performance Optimizations

1. **Concurrent Block Fetching:**
   - Default: 6 parallel requests
   - Configurable: 4-12 range
   - Per-provider concurrency limit: 3

2. **Caching Strategy:**
   - Pin all published sites (never evict)
   - Auto-cache viewed sites (7-day TTL)
   - LRU eviction when cache full
   - Manifest cache separate from blocks

3. **Provider Selection:**
   - Prefer LAN providers (lowest latency)
   - Score by: latency (40%), availability (40%), score from rendezvous (20%)
   - Failover on timeout (10s per provider)

---

## 📱 MOBILE CONSIDERATIONS (Future)

While MVP is desktop-only, design with mobile in mind:

**Mobile-Friendly Patterns:**
- Touch-friendly tap targets (44px min)
- Responsive layouts (flex, grid)
- Gesture support (swipe back/forward)
- Offline-first (essential for mobile)

**Mobile-Specific Features (Phase 2):**
- QR code sharing (scan to visit site)
- Background seeding toggle (battery consideration)
- Cellular data warnings
- Bluetooth LAN discovery

---

## 🎨 DESIGN SYSTEM

### Color Palette (PQC/Security Theme)

```typescript
const securityColors = {
  verified: '#2EB67D', // Green - signature valid
  tofu: '#F2994A', // Orange - first seen
  mismatch: '#E01E5A', // Red - key changed
  unknown: '#6B7280', // Gray - not checked
  
  // Discovery
  discovering: '#1E88E5', // Blue - searching
  found: '#2EB67D', // Green - providers found
  offline: '#F2994A', // Orange - cached only
};
```

### Typography Hierarchy

```typescript
const typography = {
  fourWords: {
    fontFamily: 'monospace',
    fontSize: '1.2rem',
    fontWeight: 600,
  },
  
  siteId: {
    fontFamily: 'monospace',
    fontSize: '0.9rem',
    color: 'text.secondary',
  },
  
  fingerprint: {
    fontFamily: 'monospace',
    fontSize: '0.85rem',
    letterSpacing: '0.05em',
  },
};
```

---

## 🚀 LAUNCH STRATEGY

### Week 1: Core Implementation
- Days 1-4: Build all components
- Day 5: Integration testing
- Weekend: Buffer for issues

### Week 2: Polish & Test
- Days 1-2: Bug fixes from testing
- Day 3: User documentation
- Day 4: Internal demo/review
- Day 5: Final polish

### Week 3: Alpha Launch
- Days 1-2: Deploy to test users (10-20 people)
- Days 3-5: Monitor, collect feedback
- Weekend: Address critical issues

### Week 4: Beta Preparation
- Days 1-3: Implement feedback
- Days 4-5: Broader beta launch

---

## 📊 METRICS TO TRACK

### Publisher Metrics
- Publications created
- Average publication time
- Failures (by phase)
- Recovery kit exports
- Name conflicts

### Viewer Metrics
- Sites viewed
- Discovery time (avg)
- Fetch time (avg)
- Cache hit rate
- TOFU decisions (trust/reject)
- Provider count distribution

### Security Metrics
- Signature verifications (pass/fail)
- TOFU first-sees
- Key mismatches detected
- User trust decisions

---

## ✅ FINAL CHECKLIST

**Before Starting Implementation:**
- [ ] Review this plan with team
- [ ] Confirm backend IPC commands are feasible
- [ ] Set up UI component storybook (optional)
- [ ] Create Figma mockups (optional, we have wireframes)

**During Implementation:**
- [ ] Create each component in isolation
- [ ] Write tests alongside code
- [ ] Test with real backend frequently
- [ ] Document as you go

**Before Alpha Release:**
- [ ] All user flows tested manually
- [ ] All error states tested
- [ ] Security review completed
- [ ] User guide written
- [ ] Keyboard shortcuts documented

---

## 🎯 ESTIMATED EFFORT BREAKDOWN

| Component | Complexity | Hours | Priority |
|-----------|------------|-------|----------|
| Shared Components | Medium | 4 | P0 |
| Publisher Step 1 | Medium | 3 | P0 |
| Publisher Step 2 | High | 4 | P0 |
| Publisher Step 3 | Medium | 3 | P0 |
| Publisher Step 4 | Medium | 2 | P0 |
| Viewer Main | High | 4 | P0 |
| Discovery & Fetch | High | 6 | P0 |
| Site Info Drawer | Medium | 3 | P1 |
| TOFU Dialog | High | 2 | P0 |
| Error Handling | Medium | 3 | P0 |
| Backend Commands | High | 8 | P0 |
| Testing | Medium | 6 | P0 |
| Polish & Bugs | Low | 4 | P1 |

**Total: 52 hours (~1.5 weeks focused work)**

With realistic buffer: **2-3 weeks to production-quality UI**

---

**This plan transforms the solid backend we built into a usable product.**

The UI will make DNS-free website publishing accessible to users while maintaining security transparency and offline-first operation.

**Ready to start implementation!** 🚀
