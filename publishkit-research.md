# PublishKit: Automating the "Last Mile" of App Publishing

## The Problem

Publishing software to app stores is a massive burden that has nothing to do with building the actual product. After finishing a working application, developers face days or weeks of overhead:

- **Legal documents**: Privacy policies, terms of service, cookie policies — all need to exist, be accurate to what the app actually does, and be hosted somewhere accessible.
- **Marketing websites**: Most stores require or strongly benefit from a dedicated landing page with screenshots, descriptions, and links.
- **Store compliance**: Each store (Chrome Web Store, App Store, Google Play, etc.) has its own checklist of requirements — icon sizes, manifest fields, permission justifications, content ratings, and more.
- **Code signing**: macOS and Windows require apps to be signed and (on macOS) notarized. This involves certificates, provisioning profiles, keychain management, and Apple's notarization pipeline. It's different every time and poorly documented.
- **Store submission**: Each store has its own API, metadata format, screenshot requirements, and review process.
- **Multi-platform distribution**: If an app could ship on iOS, Android, macOS, Windows, Linux, and the web, each platform is its own distribution lane with its own tooling.

The result: working apps sit unshipped because the publishing overhead isn't worth the developer's time. Android versions of iOS apps go unshipped. Desktop apps never make it to the Mac App Store. Linux builds never get packaged. Chrome extensions take days instead of hours to publish.

---

## What Exists Today

### Store Submission & CI/CD

| Tool | What It Does | Platforms | Limitations |
|------|-------------|-----------|-------------|
| [Fastlane](https://fastlane.tools/) | CLI for automating builds, signing, screenshots, metadata, and store submission. The gold standard for mobile. | iOS, Android | Mobile only. No legal docs, no marketing sites, no Chrome/desktop. Complex setup. |
| [Runway](https://www.runway.team/) | Release management dashboard. Coordinates rollouts, store submissions, rollbacks. Sits on top of fastlane/CI. | iOS, Android | Mobile only. Paid SaaS. Doesn't help with initial setup or compliance. |
| [Appcircle](https://appcircle.io/publish-to-stores) | Full mobile CI/CD with built-in store submission and signing. | iOS, Android, Huawei | Mobile only. |
| [Codemagic](https://codemagic.io/) | CI/CD with signing and store submission. Strong Flutter/Dart support. | iOS, Android | Mobile only. |
| [Bitrise](https://bitrise.io/) | Mobile-focused CI/CD platform with store deployment steps. | iOS, Android | Mobile only. |
| [chrome-webstore-upload-cli](https://github.com/fregante/chrome-webstore-upload-cli) | CLI to upload and publish Chrome extensions via the Web Store API. | Chrome | Chrome only. Upload/publish only — no compliance checking, no legal docs. |
| [ExtensionNinja/extension-publish](https://github.com/ExtensionNinja/extension-publish) | GitHub Action wrapping Chrome Web Store upload. | Chrome | Chrome only. Same limitations. |

### Code Signing & Notarization

| Framework | How Signing Works | Notes |
|-----------|------------------|-------|
| [Electron Forge](https://www.electronforge.io/guides/code-signing/code-signing-macos) | Uses `@electron/osx-sign` and `@electron/notarize` under the hood. Configure in `packagerConfig`. | Works well once configured. Initial setup is painful — Apple Developer account, certificate creation, keychain import, notarization credentials. |
| [electron-builder](https://www.electron.build/code-signing-mac.html) | Built-in macOS and Windows signing support via environment variables. | Similar to Forge. Supports GitHub Actions pipelines. |
| [Tauri v2](https://v2.tauri.app/distribute/sign/macos/) | Automatically signs and notarizes during `tauri build` when environment variables are set. | Simplest DX of the three. Still requires the same Apple Developer setup. |

All three frameworks support automated cross-platform builds and signing in CI (GitHub Actions, etc.), but the initial certificate/provisioning setup remains manual and error-prone.

### Linux Distribution

Three main universal package formats:

| Format | Maintained By | Notes |
|--------|--------------|-------|
| **Snap** | Canonical | Native to Ubuntu. Auto-updates. Requires Snapcraft account. Published via Snap Store. |
| **Flatpak** | Freedesktop.org | Community-driven. Published via Flathub. Sandboxed. |
| **AppImage** | Community | Portable, no installation needed. No central store — distribute the file yourself. |

Electron-builder and Tauri can both output `.deb`, `.rpm`, `.AppImage`, and Snap packages. Flatpak requires separate configuration. Each format has its own metadata, sandboxing, and permission model.

### Privacy Policy & Terms of Service Generators

| Tool | What It Generates | Pricing | Notes |
|------|------------------|---------|-------|
| [Termly](https://termly.io/) | Privacy policy, ToS, cookie policy, return policy, and more | Free tier + paid plans | Covers GDPR, CCPA, 28+ laws. Updates generators as laws change. |
| [GetTerms](https://getterms.io/) | Privacy policy + ToS + cookie policy + acceptable use policy | Free tier + paid plans | SaaS-focused. All docs generated together. |
| [TermsFeed](https://www.termsfeed.com/) | Privacy policy, ToS, EULA, cookie policy | Free tier + paid plans | Exports HTML, DOCX, plain text, Markdown. |
| [PrivacyPolicies.com](https://www.privacypolicies.com/) | Privacy policy, ToS | Free | Basic but functional. |

All of these work via guided questionnaires — the developer answers questions about their app and the tool generates documents. **None of them analyze your actual codebase** to determine what data you collect or what permissions you use.

### Marketing / Landing Page Tools

| Tool | What It Does | Cost |
|------|-------------|------|
| [Carrd](https://carrd.co/) | Simple one-page sites. Popular with indie hackers. | $19/year |
| [Unicorn Platform](https://unicornplatform.com/) | Landing pages for SaaS/apps with AI features and pre-built components. | Paid plans |
| [ShipFa.st](https://shipfa.st/) | Next.js SaaS starter kit with landing page, auth, payments. | $200 one-time |
| [Webflow](https://webflow.com/) | Full drag-and-drop website builder. Powerful but steep learning curve. | Paid plans |

These are general-purpose tools. None are integrated with app store workflows or auto-populate from your app's metadata.

### App Store Metadata & ASO

| Tool | What It Does |
|------|-------------|
| [AppDrift](https://appdrift.co/) | AI-generated app store metadata (titles, descriptions, keywords). Screenshot templates. Publishes to App Store and Google Play. |

### All-in-One Platforms (No-Code)

| Tool | What It Does | Limitation |
|------|-------------|-----------|
| [a0.dev](https://a0.dev/) | AI-powered app building with one-click publishing to App Store and Google Play. | You must build the app with their platform. Doesn't work with existing codebases. |
| [Adalo](https://www.adalo.com/) | No-code app builder with built-in store submission. | Same — no-code only. |
| [Instant Developer](https://www.instantdeveloper.com/) | Generates store-ready packages for iOS and Android. | Requires using their platform. |

These solve publishing by owning the entire app lifecycle. They don't help developers who already have a codebase.

---

## The Gap

**No single tool takes an existing developer's codebase and handles the full "publish to store" lifecycle end-to-end.** Specifically:

1. **No tool connects** code analysis to legal document generation to marketing site creation to store submission in one flow.
2. **No cross-platform store tool** handles Chrome Web Store + iOS App Store + Google Play + macOS App Store + Linux distribution from one interface.
3. **No AI-powered compliance checker** reads a codebase, determines what data it collects and what permissions it uses, and generates accurate legal documents from that analysis.
4. **No tool auto-generates** a marketing website from an app's metadata and deploys it.
5. **No signing assistant** provides reliable, repeatable instructions or automation for macOS/Windows code signing that works every time without re-learning the process.

The closest thing is **Fastlane**, which covers iOS and Android submission well, but it handles none of the legal, marketing, or compliance-checking parts, and doesn't touch Chrome, desktop, or Linux.

---

## What the Solution Would Look Like

### Core Concept

A unified system that takes an existing codebase from "working app" to "published on store" by handling everything the developer shouldn't have to think about: legal documents, marketing presence, store compliance, code signing, and submission.

### Architecture: Web Platform + CLI Hybrid

The recommended approach combines a **CLI** for local operations with a **web platform** for hosted services.

#### CLI (runs locally, interacts with the codebase)

```
publishkit init          # Scans codebase, detects platform (Chrome extension, Electron, Tauri, iOS, Android, etc.)
publishkit legal         # Generates privacy policy + ToS based on code analysis
publishkit site          # Generates and deploys a marketing site
publishkit check         # Validates project against store requirements
publishkit sign          # Automates or guides signing setup
publishkit submit        # Uploads to the relevant store
```

#### Web Platform (hosted infrastructure)

- Dashboard to manage apps, view submission status, edit marketing sites
- Hosts marketing sites, privacy policies, and ToS pages on user's custom domain or a subdomain (e.g., `yourapp.publishkit.dev`)
- Custom domain support via Cloudflare for SaaS (Custom Hostnames API) — provisions SSL for any user domain pointed at the platform
- Account management for store credentials (Apple Developer, Google Play, Chrome Web Store API keys)

#### Custom Domain Flow

1. User adds their custom domain in the dashboard
2. Platform registers the custom hostname on the PublishKit Cloudflare zone via Custom Hostnames API (provisions SSL)
3. Platform detects the user's DNS provider from nameserver lookup
4. Shows the CNAME record to add (e.g., `myapp.com` → `custom.publishkit.dev`), tailored to the detected provider's format, with a deep link to their DNS settings when possible
5. **If the user is on Cloudflare**: offer an OAuth flow to auto-add the CNAME record
6. **If not**: user adds the record manually at their DNS provider
7. User clicks "Verify" → platform checks DNS resolution
8. Status: pending → verified → live

This is the same pattern used by Resend and similar platforms. Works for any DNS provider, with a shortcut for Cloudflare users.

### Component Breakdown

#### 1. Code Scanner

Runs locally via the CLI. Analyzes the codebase to determine:

- **Platform/framework**: Chrome extension (manifest.json), Electron (package.json + electron), Tauri (tauri.conf.json), iOS (Xcode project), Android (Gradle), etc.
- **Permissions used**: Camera, location, storage, network, clipboard, notifications
- **Data collected**: Analytics SDKs, form fields, authentication flows, local storage usage, cookies
- **Third-party services**: Tracking pixels, ad networks, crash reporting, analytics platforms
- **APIs called**: External service integrations that affect privacy policy requirements

Output: A structured manifest describing what the app does, used as input for legal document generation and compliance checking.

#### 2. Legal Document Generator

Takes the code scanner output plus answers to a short questionnaire (business name, contact email, jurisdiction) and generates:

- **Privacy policy**: Accurate to what the app actually collects and shares, compliant with GDPR, CCPA, and other relevant laws
- **Terms of service**: Covers the specific type of app (extension, mobile app, desktop app, SaaS)
- **Cookie policy**: If applicable based on detected cookie/tracking usage

Documents are hosted on the web platform and accessible via the user's domain. The AI-powered generation is the differentiator — existing tools ask dozens of questions; this one reads the code.

#### 3. Marketing Site Generator

Generates a landing page from:

- App name, tagline, and description
- Screenshots and icons (pulled from store metadata or provided by developer)
- Store links (auto-generated after submission)
- Feature list
- Privacy policy and ToS links

Served as dynamic routes from the main SvelteKit app (not separate static deployments). User connects a custom domain or uses a provided subdomain (e.g., `yourapp.publishkit.dev`). Templates are Svelte components optimized for different app types (mobile app, browser extension, desktop app, SaaS). Customizable within templates (colors, fonts, copy, screenshots). Users who outgrow the templates can export as static HTML and self-host.

#### 4. Store Compliance Checker

A rules engine with per-store requirements:

- **Chrome Web Store**: Manifest V3 compliance, required fields, permission justifications, icon sizes, single-purpose policy, remote code restrictions
- **iOS App Store**: Info.plist requirements, privacy nutrition labels, App Tracking Transparency, minimum OS version, required device capabilities
- **Google Play**: Target API level, data safety section, content rating questionnaire, permission declarations, 64-bit requirement
- **macOS App Store**: Sandboxing requirements, hardened runtime, notarization
- **Linux (Snap/Flatpak)**: Desktop file, AppStream metadata, sandbox permissions

Output: A checklist with pass/fail status and actionable fix instructions for each item.

#### 5. Signing Assistant

Platform-specific automation or guided setup:

- **macOS**: Certificate creation (or import), provisioning profile management, keychain setup, `codesign` execution, `xcrun notarytool` submission. Stores configuration so subsequent builds "just work."
- **Windows**: Guidance for certificate acquisition. (Windows signing is complex enough that full automation may be out of initial scope.)
- **Android**: Keystore creation and management, signing configuration in Gradle.
- **iOS**: Certificate and provisioning profile management via App Store Connect API.

The goal: run signing once with guided setup, then it works automatically on every subsequent build.

#### 6. Store Submitter

Wraps existing store APIs:

- **Chrome Web Store**: Upload zip, set metadata, publish (via Chrome Web Store API)
- **iOS App Store**: Upload build, set metadata, submit for review (via App Store Connect API or `altool`/`xcrun`)
- **Google Play**: Upload AAB/APK, set metadata, manage tracks (via Google Play Developer API / fastlane supply)
- **macOS App Store**: Upload pkg, submit for review (via `xcrun altool` or Transporter)
- **Linux**: Upload to Snap Store (via `snapcraft`), submit to Flathub (via PR to flathub repo)

### Architecture Decision: CLI + Web Platform Hybrid

CLI for local operations (code scanning, building, signing), web platform for hosted services (marketing sites, legal docs, dashboard, domain routing).

Other options were considered and rejected:
- **CLI only**: Developer still has to host marketing sites and legal docs themselves. Defeats the purpose.
- **Claude Code / MCP integration**: Interesting but too dependent on a single tool and harder to productize.
- **Web platform only**: Security concerns with giving a platform source code access. Can't handle local signing.

### Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| CLI | TypeScript (Commander + Inquirer) | Shared types with web platform via monorepo. npm/npx distribution. Fastest dev velocity. Interactive prompts for guided flows, flags for CI/CD. |
| Web Platform | SvelteKit on Cloudflare | Simpler mental model, less boilerplate, excellent Cloudflare adapter, smaller bundles. |
| Marketing Site Hosting | SvelteKit dynamic routes | User sites are served as dynamic routes from the main SvelteKit app (not separate deployments). Subdomain or custom domain resolves to the same app, which looks up app data and renders the right template. No per-user build/deploy pipeline. |
| AI Integration | Claude API | Code analysis, legal document generation, compliance reasoning. |
| Store APIs | Direct integration | Chrome Web Store API, App Store Connect API, Google Play Developer API. |
| Domain Management | Cloudflare API | DNS record creation, SSL provisioning, domain routing. |
| Database | Postgres (e.g., Neon or Supabase) | User accounts, app metadata, submission history. |
| Auth | OAuth with Apple/Google/GitHub | Developer-friendly sign-in. |

### CLI Design

The CLI uses standard non-interactive flags for CI/CD and interactive prompts for guided setup flows. Every interactive command also accepts flags to skip prompts.

```
# Non-interactive (CI/CD, scripting)
publishkit check --platform chrome --format json
publishkit submit --platform chrome --zip ./dist.zip --silent

# Interactive (first-time setup, guided flows)
publishkit init        # asks questions, detects platform
publishkit sign        # walks through cert setup step by step
publishkit legal       # short questionnaire, then generates
```

### Marketing Site Architecture

User sites are **not** individually generated and deployed. They are dynamic routes served by the main SvelteKit app:

```
myapp.publishkit.dev/          -> marketing landing page
myapp.publishkit.dev/privacy   -> privacy policy
myapp.publishkit.dev/terms     -> terms of service
```

A SvelteKit hook reads the hostname, looks up the app in the database, and renders the appropriate template. Templates are Svelte components with different layouts per app type (mobile app, browser extension, desktop app).

Customization is available within templates (colors, fonts, copy, screenshots, section ordering). Users who outgrow the templates can export as static HTML and self-host — no lock-in.

Custom domains work via DNS: user points their domain at the platform, Cloudflare handles SSL, SvelteKit resolves the hostname.

### Monorepo Structure

```
publishkit/
  packages/
    cli/           # TypeScript CLI (Commander + Inquirer)
    web/           # SvelteKit web platform
    shared/        # Shared types, store rules, compliance definitions
```

One repo, one language (TypeScript), shared types between CLI and web.

### Complexity Assessment

| Component | Difficulty | Notes |
|-----------|-----------|-------|
| Chrome Web Store automation | **Low** | API is simple and well-documented. Existing CLI tools to reference. Good starting point. |
| Privacy policy / ToS generation | **Medium** | AI can draft accurately from code analysis. Legal review needed for template correctness. Liability is a consideration — may need "generated, not legal advice" disclaimers. |
| Marketing site generator | **Medium** | Template system + Cloudflare Pages API. Design is the main challenge, not engineering. |
| Store compliance checker | **Medium-High** | Need to encode and maintain rules for each store. Rules change over time. |
| Android signing + submission | **Medium** | Fastlane supply handles most of this. Wrapping it is straightforward. |
| Linux distribution | **Medium** | Multiple formats, each with different packaging and metadata requirements. |
| iOS/macOS signing + submission | **High** | Apple's tooling is complex. Certificates expire. Provisioning profiles are fragile. App Store Connect API is extensive. |
| Windows signing | **High** | Certificate acquisition is expensive and bureaucratic. Signing tooling varies. Deferring this is reasonable. |

### Recommended Build Order

1. **Chrome extensions** — simplest store, you've felt the pain directly, fastest path to a working product
2. **Marketing site + legal doc generation** — applies to all platforms, immediately useful
3. **iOS + Android** (via fastlane wrapping) — largest market, most demand
4. **macOS desktop** (Electron/Tauri signing) — extends naturally from iOS signing setup
5. **Linux distribution** — smaller market but underserved
6. **Windows signing** — defer until there's demand

---

## Key Insight

The individual pieces of this problem are largely solved by existing tools (fastlane, chrome-webstore-upload-cli, Termly, Carrd, etc.). What's missing is the **integration layer** — a single system that orchestrates all of them, powered by AI that can read a codebase and make intelligent decisions about what's needed. The product isn't any single feature; it's the elimination of the gap between "my app works" and "my app is published."
