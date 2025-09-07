// New Web Interactive Demo
// Demonstrates Four-Word Networking and DNS-free websites

// Word lists for generating random identities
const wordLists = {
    adjectives: ['ocean', 'forest', 'crystal', 'ancient', 'golden', 'silver', 'cosmic', 'quantum', 'digital', 'eternal', 'mystic', 'sacred', 'hidden', 'bright', 'dark', 'swift', 'bold', 'wise', 'pure', 'wild'],
    nouns: ['tree', 'star', 'moon', 'river', 'mountain', 'cloud', 'storm', 'light', 'shadow', 'phoenix', 'dragon', 'wolf', 'eagle', 'tiger', 'lion', 'bear', 'fox', 'hawk', 'dove', 'raven'],
    colors: ['blue', 'green', 'red', 'black', 'white', 'purple', 'orange', 'yellow', 'cyan', 'magenta', 'indigo', 'violet', 'crimson', 'azure', 'emerald', 'ruby', 'onyx', 'pearl', 'jade', 'amber'],
    elements: ['fire', 'water', 'earth', 'air', 'metal', 'wood', 'ice', 'thunder', 'lightning', 'mist', 'rain', 'snow', 'wind', 'stone', 'crystal', 'sand', 'dust', 'ash', 'smoke', 'steam']
};

// Demo state
let demoState = {
    identity: 'black tree fish river',
    publicKey: '0x7f3a9b2c8e4d1a5f',
    websiteRoot: null,
    currentStep: 0
};

// Initialize demo on page load
document.addEventListener('DOMContentLoaded', () => {
    initializeDemo();
});

function initializeDemo() {
    // Set initial identity display
    const identityInput = document.getElementById('demo-identity');
    if (identityInput) {
        identityInput.value = demoState.identity;
    }
    
    // Initialize website root display
    updateWebsiteRootDisplay();
    
    // Add event listeners
    setupEventListeners();
    
    // Animate word chips
    animateWordChips();
}

function setupEventListeners() {
    // Randomize button
    const randomizeBtn = document.querySelector('button[onclick="randomizeIdentity()"]');
    if (randomizeBtn) {
        randomizeBtn.onclick = randomizeIdentity;
    }
    
    // Publish button
    const publishBtn = document.querySelector('button[onclick="publishSite()"]');
    if (publishBtn) {
        publishBtn.onclick = publishSite;
    }
    
    // Visit button
    const visitBtn = document.querySelector('button[onclick="visitSite()"]');
    if (visitBtn) {
        visitBtn.onclick = visitSite;
    }
    
    // Link to group button
    const linkBtn = document.querySelector('button[onclick="linkToGroup()"]');
    if (linkBtn) {
        linkBtn.onclick = linkToGroup;
    }
    
    // Reset button
    const resetBtn = document.querySelector('button[onclick="resetDemo()"]');
    if (resetBtn) {
        resetBtn.onclick = resetDemo;
    }
}

function randomizeIdentity() {
    // Generate random four words
    const words = [
        getRandomWord(wordLists.colors),
        getRandomWord(wordLists.nouns),
        getRandomWord(wordLists.adjectives),
        getRandomWord(wordLists.elements)
    ];
    
    demoState.identity = words.join(' ');
    demoState.publicKey = generateMockPublicKey();
    demoState.websiteRoot = null;
    
    // Update display
    const identityInput = document.getElementById('demo-identity');
    if (identityInput) {
        identityInput.value = demoState.identity;
        identityInput.classList.add('identity-transition');
        setTimeout(() => identityInput.classList.remove('identity-transition'), 300);
    }
    
    // Update public key display
    const pkDisplay = document.querySelector('.mono-text');
    if (pkDisplay) {
        pkDisplay.textContent = `Public Key: ${demoState.publicKey}...`;
    }
    
    // Reset website root
    updateWebsiteRootDisplay();
    
    // Reset demo steps if needed
    if (demoState.currentStep > 0) {
        resetDemo();
    }
}

function getRandomWord(list) {
    return list[Math.floor(Math.random() * list.length)];
}

function generateMockPublicKey() {
    const chars = '0123456789abcdef';
    let key = '0x';
    for (let i = 0; i < 16; i++) {
        key += chars[Math.floor(Math.random() * chars.length)];
    }
    return key;
}

function generateMockWebsiteRoot() {
    const chars = '0123456789abcdef';
    let root = '';
    for (let i = 0; i < 64; i++) {
        root += chars[Math.floor(Math.random() * chars.length)];
    }
    return root;
}

function updateWebsiteRootDisplay() {
    const rootDisplay = document.getElementById('website-root');
    if (rootDisplay) {
        rootDisplay.textContent = demoState.websiteRoot || 'none';
        if (demoState.websiteRoot) {
            rootDisplay.classList.add('website-root-active');
        } else {
            rootDisplay.classList.remove('website-root-active');
        }
    }
}

async function publishSite() {
    const output = document.getElementById('step1-output');
    const step2 = document.getElementById('step2');
    
    // Show loading state
    if (output) {
        output.innerHTML = '<div class="loading-dots">Publishing site<span>.</span><span>.</span><span>.</span></div>';
    }
    
    // Simulate publishing delay
    await sleep(1500);
    
    // Generate website root
    demoState.websiteRoot = generateMockWebsiteRoot();
    
    // Generate canonical bytes simulation
    const canonicalBytes = generateCanonicalBytes();
    
    // Show success
    if (output) {
        output.innerHTML = `
            <div class="demo-success">
                <div class="success-icon">✓</div>
                <div class="success-details">
                    <p><strong>Website published!</strong></p>
                    <p class="mono-text small">Root: ${demoState.websiteRoot.substring(0, 16)}...</p>
                    <p class="mono-text small">Canonical: ${canonicalBytes}</p>
                    <p class="demo-note">Signed with ML-DSA, distributed via DHT</p>
                </div>
            </div>
        `;
    }
    
    // Update website root display
    updateWebsiteRootDisplay();
    
    // Show next step
    if (step2) {
        step2.style.display = 'block';
        step2.classList.add('step-enter');
    }
    
    demoState.currentStep = 1;
}

async function visitSite() {
    const output = document.getElementById('step2-output');
    const step3 = document.getElementById('step3');
    
    // Show loading state
    if (output) {
        output.innerHTML = '<div class="loading-dots">Resolving identity<span>.</span><span>.</span><span>.</span></div>';
    }
    
    await sleep(1000);
    
    // Show site preview
    if (output) {
        output.innerHTML = `
            <div class="site-preview">
                <div class="browser-bar">
                    <div class="browser-url">communitas://${demoState.identity.replace(/ /g, '-')}</div>
                </div>
                <div class="site-content">
                    <h4>Welcome to ${demoState.identity}'s Site</h4>
                    <p>This site is hosted on the distributed network.</p>
                    <p class="mono-text small">No DNS · No servers · Quantum-proof</p>
                    <div class="site-meta">
                        <span>📍 Resolved via Four-Word address</span>
                        <span>🔒 ML-DSA verified</span>
                        <span>🌐 Content-addressed</span>
                    </div>
                </div>
            </div>
        `;
    }
    
    // Show next step
    if (step3) {
        step3.style.display = 'block';
        step3.classList.add('step-enter');
    }
    
    demoState.currentStep = 2;
}

async function linkToGroup() {
    const output = document.getElementById('step3-output');
    
    // Show loading state
    if (output) {
        output.innerHTML = '<div class="loading-dots">Discovering links<span>.</span><span>.</span><span>.</span></div>';
    }
    
    await sleep(1000);
    
    // Generate some mock group identities
    const groups = [
        { identity: 'quantum labs research collective', type: 'Organization' },
        { identity: 'secure mesh network alliance', type: 'Group' },
        { identity: 'privacy first developers guild', type: 'Community' }
    ];
    
    // Show links
    if (output) {
        output.innerHTML = `
            <div class="links-browser">
                <h4>Linked Sites</h4>
                <div class="link-list">
                    ${groups.map(group => `
                        <div class="link-item">
                            <div class="link-identity">${group.identity}</div>
                            <div class="link-type">${group.type}</div>
                            <div class="link-status">✓ Verified</div>
                        </div>
                    `).join('')}
                </div>
                <div class="demo-note">
                    Each identity owns its website. Links are cryptographically verified.
                    No central authority needed.
                </div>
            </div>
        `;
    }
    
    demoState.currentStep = 3;
}

function resetDemo() {
    // Reset state
    demoState.currentStep = 0;
    demoState.websiteRoot = null;
    
    // Clear outputs
    const outputs = ['step1-output', 'step2-output', 'step3-output'];
    outputs.forEach(id => {
        const element = document.getElementById(id);
        if (element) element.innerHTML = '';
    });
    
    // Hide steps
    const steps = ['step2', 'step3'];
    steps.forEach(id => {
        const element = document.getElementById(id);
        if (element) {
            element.style.display = 'none';
            element.classList.remove('step-enter');
        }
    });
    
    // Update website root display
    updateWebsiteRootDisplay();
}

function generateCanonicalBytes() {
    const dst = 'saorsa-identity:website_root:v1';
    const pk = demoState.publicKey;
    const root = demoState.websiteRoot ? demoState.websiteRoot.substring(0, 8) : '00000000';
    return `${dst}|${pk}|${root}...`;
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function animateWordChips() {
    const chips = document.querySelectorAll('.word-chip');
    chips.forEach((chip, index) => {
        chip.style.animationDelay = `${index * 0.1}s`;
    });
}

// Add some CSS classes dynamically for animations
const style = document.createElement('style');
style.textContent = `
    .identity-transition {
        animation: identityPulse 0.3s ease-out;
    }
    
    @keyframes identityPulse {
        0% { transform: scale(1); }
        50% { transform: scale(1.02); background: rgba(99, 102, 241, 0.1); }
        100% { transform: scale(1); }
    }
    
    .website-root-active {
        color: #10b981 !important;
        font-weight: 600;
    }
    
    .loading-dots span {
        animation: dotPulse 1.4s infinite;
    }
    
    .loading-dots span:nth-child(2) {
        animation-delay: 0.2s;
    }
    
    .loading-dots span:nth-child(3) {
        animation-delay: 0.4s;
    }
    
    @keyframes dotPulse {
        0%, 60%, 100% { opacity: 0.3; }
        30% { opacity: 1; }
    }
    
    .step-enter {
        animation: slideIn 0.3s ease-out;
    }
    
    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateY(10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }
    
    .demo-success {
        display: flex;
        gap: 1rem;
        padding: 1rem;
        background: linear-gradient(135deg, rgba(16, 185, 129, 0.1), rgba(99, 102, 241, 0.1));
        border-radius: 8px;
        border: 1px solid rgba(99, 102, 241, 0.2);
    }
    
    .success-icon {
        font-size: 2rem;
        color: #10b981;
    }
    
    .success-details p {
        margin: 0.25rem 0;
    }
    
    .mono-text.small {
        font-size: 0.875rem;
        opacity: 0.8;
    }
    
    .demo-note {
        font-size: 0.875rem;
        color: #6b7280;
        margin-top: 0.5rem;
    }
    
    .site-preview {
        border: 1px solid #e5e7eb;
        border-radius: 8px;
        overflow: hidden;
        background: white;
    }
    
    .browser-bar {
        background: #f3f4f6;
        padding: 0.5rem 1rem;
        border-bottom: 1px solid #e5e7eb;
    }
    
    .browser-url {
        font-family: 'Monaco', 'Courier New', monospace;
        font-size: 0.875rem;
        color: #6b7280;
    }
    
    .site-content {
        padding: 1.5rem;
    }
    
    .site-content h4 {
        margin: 0 0 1rem 0;
        color: #111827;
    }
    
    .site-content p {
        margin: 0.5rem 0;
        color: #4b5563;
    }
    
    .site-meta {
        display: flex;
        gap: 1rem;
        margin-top: 1rem;
        padding-top: 1rem;
        border-top: 1px solid #e5e7eb;
        font-size: 0.875rem;
        color: #6b7280;
    }
    
    .links-browser {
        background: white;
        border: 1px solid #e5e7eb;
        border-radius: 8px;
        padding: 1.5rem;
    }
    
    .links-browser h4 {
        margin: 0 0 1rem 0;
        color: #111827;
    }
    
    .link-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }
    
    .link-item {
        display: grid;
        grid-template-columns: 1fr auto auto;
        gap: 1rem;
        padding: 0.75rem;
        background: #f9fafb;
        border: 1px solid #e5e7eb;
        border-radius: 6px;
        align-items: center;
    }
    
    .link-identity {
        font-weight: 500;
        color: #111827;
    }
    
    .link-type {
        font-size: 0.875rem;
        color: #6b7280;
        background: #f3f4f6;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
    }
    
    .link-status {
        color: #10b981;
        font-size: 0.875rem;
    }
`;

document.head.appendChild(style);