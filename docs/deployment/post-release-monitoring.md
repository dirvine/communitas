# Post-Release Monitoring

Procedures for monitoring Communitas after a production release.

## Metrics to Monitor

### Download & Adoption
| Metric | Source | Target |
|--------|--------|--------|
| Downloads | GitHub Releases API | Track trends |
| Active installs | Update check frequency | Track growth |
| Platform distribution | DMG downloads | Intel vs ARM balance |

### Quality Indicators
| Metric | Source | Target |
|--------|--------|--------|
| Crash reports | User reports / future telemetry | < 0.1% sessions |
| Issue volume | GitHub Issues | Decreasing trend |
| Issue severity | Issue labels | No P0/P1 regressions |

### User Feedback
| Metric | Source | Target |
|--------|--------|--------|
| Issue reports | GitHub Issues | Respond within SLA |
| Feature requests | GitHub Issues | Track for roadmap |
| Community sentiment | Social channels | Positive |

## Monitoring Tools

### GitHub API Queries

Check release downloads:
```bash
curl -s https://api.github.com/repos/maidsafe/communitas/releases/latest \
  | jq '.assets[] | {name, download_count}'
```

Check open issues by label:
```bash
gh issue list --repo maidsafe/communitas --label "bug" --state open
```

### Update Check Monitoring

The auto-updater checks `update.json` from GitHub Releases. Monitor:
- Download frequency (via GitHub Releases stats)
- Geographic distribution (if CDN used)
- Error rates (via user reports)

## Issue Triage Process

### Priority Levels

| Priority | Definition | Examples |
|----------|------------|----------|
| **P0** | App unusable | Crash on launch, data loss, security vulnerability |
| **P1** | Core feature broken | Can't send messages, files won't upload |
| **P2** | Feature degraded | Slow performance, minor UI bugs |
| **P3** | Enhancement | New feature requests, nice-to-haves |

### Triage Workflow

1. **New Issue Arrives**
   - Read issue description and reproduction steps
   - Check if duplicate of existing issue
   - Assign priority label

2. **P0/P1 Issues**
   - Immediate notification to team
   - Begin investigation within SLA
   - Create branch for fix
   - Fast-track review and release

3. **P2/P3 Issues**
   - Add to backlog
   - Schedule for next sprint/release
   - Acknowledge to reporter

## Response Time SLAs

| Priority | Acknowledgment | Resolution Target |
|----------|----------------|-------------------|
| P0 | 4 hours | 24 hours (hotfix) |
| P1 | 24 hours | 1 week |
| P2 | 48 hours | Next release |
| P3 | 1 week | Roadmap |

## Hotfix Process

When a P0 or critical P1 issue is discovered:

### 1. Verify the Issue
```bash
# Reproduce on clean install
# Check logs for error details
# Confirm scope of impact
```

### 2. Create Hotfix Branch
```bash
git checkout -b hotfix/v1.0.1
```

### 3. Fix and Test
```bash
# Make minimal fix
cargo test --workspace
./scripts/run-full-regression.sh --quick
```

### 4. Version Bump
```bash
# Update version to 1.0.1
# Update CHANGELOG.md
```

### 5. Release
```bash
git tag v1.0.1
git push origin v1.0.1
# Release workflow runs automatically
```

### 6. Communicate
- Update GitHub issue with fix
- Release notes on GitHub Release
- Notify users via appropriate channels

## Success Metrics

### Week 1 Post-Release
- [ ] Zero P0 issues
- [ ] < 5 P1 issues
- [ ] Download count tracking enabled
- [ ] Support channels active

### Month 1 Post-Release
- [ ] No hotfixes required
- [ ] User feedback collected
- [ ] Performance metrics baseline established
- [ ] Roadmap updated based on feedback

### Quarterly Review
- [ ] Release cadence meeting targets
- [ ] User adoption growing
- [ ] Issue volume stable or decreasing
- [ ] SLAs consistently met

## Communication Plan

### Internal
- Slack channel for release monitoring
- Daily standup updates during first week
- Weekly summary thereafter

### External
- GitHub Discussions for announcements
- Issue responses within SLA
- Release notes for each version

## Escalation Path

| Level | Scope | Contact |
|-------|-------|---------|
| 1 | Individual contributor | Assigned developer |
| 2 | Team | Team lead |
| 3 | Critical | Project lead |
| 4 | Emergency | Executive team |

## Post-Mortem Process

For any P0 incident or significant outage:

1. **Document** - Create incident report within 48 hours
2. **Timeline** - Record sequence of events
3. **Root Cause** - Identify underlying issue
4. **Impact** - Quantify affected users/scope
5. **Actions** - Define preventive measures
6. **Share** - Publish sanitized learnings

## Rollback Procedure

If a release causes critical issues:

### 1. Assess Impact
- How many users affected?
- Is data at risk?
- Can workaround be provided?

### 2. Decide: Hotfix vs Rollback
- Hotfix if fix is simple and low-risk
- Rollback if fix is complex or risky

### 3. Execute Rollback
```bash
# Mark current release as deprecated
gh release edit v1.0.0 --prerelease

# Point update.json to previous stable version
# Users will update to last known good version
```

### 4. Communicate
- GitHub Issue update
- Release notes explaining rollback
- ETA for fixed release

## Future Enhancements

Planned monitoring improvements:

- [ ] Crash reporting integration (Sentry/similar)
- [ ] Anonymous usage telemetry (opt-in)
- [ ] Automated issue triage
- [ ] Performance monitoring dashboard
- [ ] User satisfaction surveys
