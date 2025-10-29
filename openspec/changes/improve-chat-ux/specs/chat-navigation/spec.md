# Spec: Chat Navigation Links

## ADDED Requirements

### Application header SHALL include chat navigation link

**ID**: `chat-nav-001`

#### Scenario: User views any page

**Given** user is on any page (landing, dashboard, docs, etc.)
**When** viewing the header
**Then** "Chat" link is visible after "API Docs"
**And** link includes MessageSquare icon
**And** clicking navigates to `/chat` page

**Acceptance Criteria**:
- Visible on all pages
- Consistent with other nav link styling (ghost button)
- Icon and label for clarity
- Positioned after "API Docs", before theme controls

### Landing page hero SHALL include chat button for authenticated users

**ID**: `chat-nav-002`

#### Scenario: Authenticated user views landing page

**Given** user is authenticated
**When** viewing landing page hero section
**Then** "Try Chat" button is visible
**And** button is styled as outline variant
**And** clicking navigates to `/chat` page

**Acceptance Criteria**:
- Only shown to authenticated users
- Positioned next to primary CTA ("Dashboard" / "Get Started")
- Outline variant to differentiate from primary action
- MessageSquare icon for recognition

#### Scenario: Unauthenticated user views landing page

**Given** user is not authenticated
**When** viewing landing page hero section
**Then** "Try Chat" button is NOT displayed
**And** only login/register buttons shown

**Acceptance Criteria**:
- Chat link hidden for non-authenticated users
- No broken links or auth redirects from landing
- Clear path to auth before accessing chat

### Landing page features grid SHALL include chat feature card

**ID**: `chat-nav-003`

#### Scenario: User browses landing page features

**Given** user views landing page
**When** scrolling to features section
**Then** "AI Chat Assistant" feature card is visible
**And** card describes chat capabilities
**And** card is visually consistent with other feature cards

**Acceptance Criteria**:
- Title: "AI Chat Assistant"
- Description highlights: Multiple models, streaming, session management
- Icon: MessageSquare or appropriate chat icon
- Positioned naturally in features grid (suggest after "Beautiful Themes")
- Lists 3 key features with checkmarks

### Chat navigation elements SHALL respect feature flag

**ID**: `chat-nav-004`

#### Scenario: Chat feature is disabled

**Given** `FEATURE_CHAT_ENABLED=false` in environment
**When** application loads
**Then** chat navigation links are hidden
**And** direct `/chat` navigation shows disabled message

**Acceptance Criteria**:
- All chat nav elements check feature flag
- Graceful degradation if feature disabled
- No broken links or UI artifacts

## MODIFIED Requirements

None - nav structure remains unchanged, only adding links.

## REMOVED Requirements

None - no existing chat navigation to remove.

## Related Capabilities

- **backend-model-filtering**: Chat must be functional for nav to be useful
- **auto-title-generation**: Improved UX makes chat worth promoting

## UI/UX Guidelines

### Header Link Placement
```tsx
<Link href="/docs">
  <Button variant="ghost" size="sm" className="gap-2">
    <BookOpen className="h-4 w-4" />
    Docs
  </Button>
</Link>
<Link href="/api-docs">
  <Button variant="ghost" size="sm" className="gap-2">
    <Code2 className="h-4 w-4" />
    API Docs
  </Button>
</Link>
{/* NEW */}
<Link href="/chat">
  <Button variant="ghost" size="sm" className="gap-2">
    <MessageSquare className="h-4 w-4" />
    Chat
  </Button>
</Link>
<ThemeSelector />
<ThemeToggle />
<UserMenu />
```

### Hero Button Placement
```tsx
<div className="flex flex-col sm:flex-row items-center justify-center gap-4 pt-4">
  {isAuthenticated ? (
    <>
      <Link href="/dashboard">
        <Button size="lg" className="gap-2">
          Go to Dashboard
          <ArrowRight className="h-4 w-4" />
        </Button>
      </Link>
      {/* NEW */}
      <Link href="/chat">
        <Button size="lg" variant="outline" className="gap-2">
          <MessageSquare className="h-4 w-4" />
          Try Chat
        </Button>
      </Link>
    </>
  ) : (
    <Link href="/register">
      <Button size="lg" className="gap-2">
        Get Started Free
        <ArrowRight className="h-4 w-4" />
      </Button>
    </Link>
  )}
</div>
```

### Feature Card Content
```tsx
<Card className="border-2 hover:border-primary/50 transition-colors">
  <CardHeader>
    <MessageSquare className="h-10 w-10 text-primary mb-2" />
    <CardTitle>AI Chat Assistant</CardTitle>
    <CardDescription>
      Multi-model LLM chat with intelligent conversation management and streaming responses
    </CardDescription>
  </CardHeader>
  <CardContent>
    <ul className="space-y-2 text-sm">
      <li className="flex items-center gap-2">
        <CheckCircle2 className="h-4 w-4 text-primary" />
        <span>Multiple AI models (Llama, GPT, Grok)</span>
      </li>
      <li className="flex items-center gap-2">
        <CheckCircle2 className="h-4 w-4 text-primary" />
        <span>Real-time streaming responses</span>
      </li>
      <li className="flex items-center gap-2">
        <CheckCircle2 className="h-4 w-4 text-primary" />
        <span>Organized session management</span>
      </li>
    </ul>
  </CardContent>
</Card>
```
