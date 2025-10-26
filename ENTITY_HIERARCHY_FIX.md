# Entity Creation Hierarchy Fix

## Issue
The + button menu was showing incorrect options that didn't respect the entity hierarchy.

## Correct Entity Hierarchy

### Personal Space
Users can:
- **Add Contact** - Add a personal contact
- **Create Group** - Create a personal group
- **Create Organisation** - Create a new organization

### Organization Space
Within an organization, users can:
- **Add Member** - Add a member to the organization (not "Add Contact")
- **Create Channel** - Create a communication channel within the org
- **Create Project** - Create a project within the org  
- **Create Group** - Create a group within the org

### All (Global View)
Shows combined options:
- Add Contact
- Create Group
- Create Organisation

## Changes Made

### File: `src/components/prototype/ModernShellPrototype.tsx`

#### Organization Scope (`scopeFilter === 'organization'`)
**Before:**
- Create Channel
- Create Project
- Create Group
- ❌ Add Contact (incorrect - contacts are personal)

**After:**
- ✅ **Add Member** (first option - primary action)
- Create Channel
- Create Project
- Create Group

#### Personal Scope (`scopeFilter === 'personal'`)
**Before:**
- Add Contact
- Create Group
- ❌ Missing: Create Organisation

**After:**
- Add Contact
- Create Group
- ✅ **Create Organisation** (added)

## Testing
1. Switch to Personal space → + button should show: Add Contact, Create Group, Create Organisation
2. Switch to an Organization → + button should show: Add Member, Create Channel, Create Project, Create Group
3. The "Add Member" option in org context correctly adds members to the organization

## Implementation Notes
- The menu is context-aware based on the `scopeFilter` state
- "Add Member" vs "Add Contact" terminology distinguishes organizational members from personal contacts
- Organization creation is only available in personal/all scope (you create orgs from personal space)
- Channels, Projects can only be created within an organization context
