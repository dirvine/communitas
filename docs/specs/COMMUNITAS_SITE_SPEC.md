# COMMUNITAS_SITE_SPEC.md — Next‑Gen **communitas.life**

**Owner:** Saorsa Labs Ltd  
**Product:** Communitas (desktop + headless node)  
**Scope:** Public website at `communitas.life` (GitHub Pages, `docs/`)  
**Objective:** Replace the current informative site with a compelling, vision‑led launch site that matches the impact of **saorsalabs.com** while clearly demonstrating the **New Web**: Four‑Word Networking, DNS‑free websites per entity, virtual disks, quantum‑secure comms and storage, and downloadable binaries.

---

## 1) Rationale and grounding

- Current site already presents the pillars and CTAs (Inter font, hero, features, tech grid, GitHub/Agents links)【125†L13-L19】【125†L37-L49】【125†L84-L100】【125†L132-L166】.
- Messaging must align with the Agents/Core APIs that enable DNS‑free websites and FWN identity, publishing flows, entity‑scoped disks, and PQC throughout【122†L13-L22】【122†L51-L56】【122†L63-L71】.
- Differentiator vs. SaaS: **every person, group, project, organisation is both a collaboration space _and_ a website**, addressed by four words; no central DNS or servers【125†L95-L113】【125†L106-L114】.

> Principle: visitors should grasp _in 10 seconds_ that Communitas = Slack/WhatsApp/Dropbox/Zoom functionality **without** servers, accounts, or DNS — and that **websites are identities**.

---

## 2) Target audiences

- **Builders & privacy advocates:** developers, cryptography‑aware users, decentralisation communities.  
- **Teams/organisations** needing private comms and publishing.  
- **Agents/automation** authors (human/AI) integrating via the AGENTS API【122†L24-L31】.

Primary tasks:
1) Download binaries.  
2) Understand the “New Web” (FWN + DNS‑free sites).  
3) Skim APIs then jump to repos.

---

## 3) Brand voice

- Clear, sovereign, technical. Short declaratives. No filler.  
- Mirror the Saorsa Labs heroic style, but make Communitas more demonstrative (live glances, animated diagrams).

Tone anchors:
- “**The People’s Internet**. No DNS. No gatekeepers. **Quantum‑proof**.”
- “Your identity is **four words**. Your work **is** your website.”
- “Peers only. **No servers.** No backdoors.”

---

## 4) Information architecture (single‑page + 2 subpages)

**/ (index.html)**  
1. **Hero** — value prop, binary download CTA.  
2. **Vision** — five‑pillars band (Imagine, Dream, Demand, Publish, Welcome)【125†L55-L79】.  
3. **Core innovations** — 5 blocks: Four‑Word Networking; DNS‑free Websites; Virtual Disks per Entity; Quantum Security; Shared Comms (messaging, channels, calls).  
4. **The New Web** — explainer + interactive demo (FWN → website).  
5. **Tech stack** — Saorsa Core, Rust, QUIC, CRDTs, Agents API【125†L139-L156】.  
6. **Get Involved** — GitHub, APIs, Community/Contact.

**/agents.html**  
- High‑level API overview with deep links to `docs/AGENTS_API.md`, highlighting key surfaces: identity claim, website publish/update, group membership, channel messaging, disks【122†L33-L59】【122†L63-L71】.

**/agents-core.html**  
- Pointers to `../saorsa-core/AGENTS_API.md` and the identity/website root canonical bytes (DNS‑free binding)【122†L51-L56】.

---

## 5) Hero section (new)

**Headline:** The People’s Internet. **No DNS. No Servers. Quantum‑Proof.**  
**Sub:** Communitas is WhatsApp + Dropbox + Slack + Zoom — **local‑first**, owner‑controlled. **Every entity is a website.**  
**CTAs:**  
- **Download Binaries** → GitHub Releases  
- **See the New Web** → jumps to “The New Web” demo  
- **Explore APIs** → /agents.html

Visual: abstract constellation of **four words** linking to glowing nodes; subtle “data shards” flow to peers.

---

## 6) Core innovations (section content and visuals)

### 6.1 Four‑Word Networking (FWN)
- Human‑readable, checksum‑safe, phishing‑resistant identities.  
- Replace DNS: **no registration, no renewal, no central authority**.  
- Link sites and people directly by words.  
**Visual:** 4 chip‑like word capsules connected by arcs.

### 6.2 DNS‑Free Websites per Entity
- Any identity publishes a **website_root** bound by ML‑DSA canonical bytes (no DNS)【122†L51-L56】.  
- Sites are content‑addressed, FEC‑sharded, optionally encrypted.  
**Visual:** Identity → signed “root” → site manifest animation.

### 6.3 Virtual Disks per Entity
- Private/Public/Shared “virtual disks” with Reed‑Solomon FEC; each disk can host content and a website【125†L106-L109】.  
**Visual:** Vault morphs into a minimal site preview.

### 6.4 Quantum‑Resistant Security
- ML‑KEM / ML‑DSA end‑to‑end for storage, comms, identities (already described on current page)【125†L100-L103】.  
**Visual:** lattice shield overlay.

### 6.5 Shared Communications
- Channels, threads, reactions, mentions; WebRTC‑over‑QUIC for calls; **no servers**.  
**Visual:** chat bubbles → screen share glyphs → “peer ↔ peer”.

---

## 7) “The New Web” interactive demo

**Goal:** Prove the mental model in 30 seconds.

**UI:** Split panel  
Left: Choose an identity: `black tree fish river` (pre‑filled example).  
Right:  
- Step 1: *Publish site* → shows “manifest saved, signed update” (animated).  
- Step 2: *Visit via Four Words* → resolves to site preview (no DNS).  
- Step 3: *Link to a Group site* → click to traverse to another identity’s site (links validated by identity).

**Implementation sketch (client‑only demo):**
- Pre‑baked JSON manifests in `/demo/` to simulate DHT fetch.  
- Small TS module renders “canonical bytes” string for display only.  
- Glossary tooltip: FWN, website_root, ML‑DSA.

---

## 8) Visual design system

- **Typography:** Inter (300–800)【125†L13-L17】.  
- **Palette:** White canvas; charcoal text; accents in blue/green (Saorsa theme).  
- **Iconography:** Minimal line icons; micro‑animations on hover.  
- **Motion:** 200–300ms transitions, easing out. **No parallax.**  
- **Accessibility:** WCAG AA colour contrast; focus rings; reduced‑motion respect.

---

## 9) Content blocks (ready‑to‑paste copy)

### 9.1 Hero
> **The People’s Internet. No DNS. No Servers. Quantum‑Proof.**  
> Communitas is WhatsApp + Dropbox + Slack + Zoom — **local‑first** and owner‑controlled.  
> Every person, group, project and organisation gets **four words** and a **website**.  
**[Download Binaries]** · **[See the New Web]** · **[Explore APIs]**

### 9.2 Vision band (5 tiles)  
Reuse current *Imagine / Dream / Demand / Publish / Welcome* with tightened copy【125†L55-L79】.

### 9.3 Core innovations (short blurbs)  
- **Four‑Word Networking:** Addresses people and services in plain words. No DNS. No phishing.  
- **DNS‑Free Websites:** Publish and link sites by identity; content is cryptographically bound.  
- **Virtual Disks:** Private/public/shared storage per entity; shard and sync across peers.  
- **Quantum Security:** ML‑KEM + ML‑DSA end‑to‑end. Future‑safe by design.  
- **Shared Comms:** Channels, threads, calls, screenshare — all peer‑to‑peer.

### 9.4 Tech grid (tight)【125†L139-L156】  
- **Rust performance** · **CRDT collab (Yjs)** · **Trust‑weighted DHT** · **Agent‑friendly APIs**.

### 9.5 CTA panel  
“**Ready to take back control?**” → **GitHub**, **Explore APIs**, **Download**【125†L158-L166】.

---

## 10) Components and sections (developer notes)

- Nav: Home, New Web, Technology, Agents, GitHub, Download.  
- Hero: h1 + sub + 2 primary CTAs.  
- Vision band: 5 equal cards, mobile carousel.  
- Innovations: 5 rows, each with icon, h3, paragraph, micro‑diagram.  
- New Web demo: controlled state machine (step indicators + reset).  
- Tech grid: 4 cards (Rust, CRDTs, DHT, APIs).  
- Footer: Resources/Tech/Legal (keep current links)【125†L170-L211】.

---

## 11) Implementation details

**Stack:** Keep plain HTML/CSS/TS for Pages. Use existing build.  
**CSS:** Single `styles.css` with CSS variables; prefers‑reduced‑motion.  
**Assets:** `/assets/` SVGs for icons; `/demo/` JSON for fake manifests.  
**Performance:**  
- Preload Inter; defer noncritical JS; compress SVGs.  
- Lighthouse ≥ 95 desktop, ≥ 90 mobile.

**Security/Privacy text:** add link to short privacy statement (no trackers).

**Download buttons:** point to Releases. If release is prebuilt by workflows, show platform chips (macOS, Windows, Linux).

---

## 12) Content wiring to AGENTS/Core (for legitimacy)

On **/agents.html**, mirror these flows from the docs:  
- **Identity claim + four words** → quick snippet.  
- **Website publish + identity update** → canonical bytes + signature path (ML‑DSA)【122†L51-L56】.  
- **Channel send/subscribe** and **Disk write/read** examples【122†L63-L71】.  
Link back to `docs/AGENTS_API.md` (Communitas) and `../saorsa-core/AGENTS_API.md` (Core).

---

## 13) SEO & social

- `<title>` and `<meta>` already present; keep but update copy for hero promise【125†L6-L12】【125†L37-L44】.  
- OpenGraph image: abstract four‑word constellation.  
- Description: *“Local‑first collaboration with Four‑Word Networking and DNS‑free websites. Post‑quantum security. No servers.”*

---

## 14) Acceptance criteria

- Homepage communicates **4‑word identity**, **DNS‑free sites**, **virtual disks**, **quantum security**, **shared comms** at a glance.  
- “New Web” demo works offline (static JSON).  
- Clear download CTAs.  
- Links to repos and Agents/Core docs.  
- Styles and motion meet accessibility.  
- Lighthouse targets met.

---

## 15) Tasks (checklist)

- [ ] Update **hero** copy + CTAs.  
- [ ] Add **Core Innovations** section with 5 blocks + SVGs.  
- [ ] Implement **New Web demo** (static simulation).  
- [ ] Tighten **Tech grid** copy + icons.  
- [ ] Footer: add **Privacy** entry and keep Legal/Links.  
- [ ] Create **/agents.html** and **/agents-core.html** stubs with deep links and short examples.  
- [ ] Add **assets/** (SVG icons, OG image).  
- [ ] Add **demo/** JSON and small TS demo handler.  
- [ ] Update `styles.css` with variables + reduced‑motion.  
- [ ] Validate WCAG; run Lighthouse.  
- [ ] Verify deploy via Pages workflow.

---

## 16) Example snippets

**Hero CTA block (HTML):**
```html
<div class="hero-ctas">
  <a class="btn btn-primary" href="https://github.com/dirvine/communitas/releases">Download Binaries</a>
  <a class="btn btn-secondary" href="#new-web">See the New Web</a>
  <a class="btn" href="agents.html">Explore APIs</a>
</div>
```

**New Web demo (outline TS):**
```ts
type Identity = { words: string; pk: string; websiteRoot?: string };
const demoId: Identity = { words: "black tree fish river", pk: "…hex…" }; // demo

function publishSite(id: Identity, rootHex: string) {
  // simulate canonical bytes / signature path
  const dst = "saorsa-identity:website_root:v1";
  const canonical = `${dst}|${id.pk}|${rootHex}`;
  return { canonical, receipt: { rootHex, ts: Date.now() } };
}
```

---

## 17) Deployment

- Same GitHub Pages pipeline.  
- Place all assets under `docs/`.  
- Verify `CNAME` remains `communitas.life`【9†L1-L1】.

---

## 18) Post‑launch metrics

- Downloads → Releases click‑through rate.  
- Time‑on‑page at “New Web” section.  
- GitHub stars/watch and issues created.  
- Newsletter / updates sign‑ups (if added).

---

## 19) Legal and trust

- Dual licence links remain (AGPL + Commercial)【125†L200-L205】.  
- Add compact Privacy link (no analytics, no trackers).

---

## 20) Summary

This spec upgrades `communitas.life` from a descriptive page to a **movement‑starter** that demonstrates the **New Web** with actionable proof (demo), sharp visuals, and honest CTAs tied directly to downloads and APIs. It remains fully static and deployable via GitHub Pages.

