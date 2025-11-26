# Agent Tune Feature Implementation

## Overview
Implemented a new "Tune" feature for agents that allows users to add additional requirements to an agent's system prompt without completely rewriting it.

## Changes Made

### 1. New Page: AgentTunePage.tsx
Created `frontend/src/pages/AgentTunePage.tsx` with:
- Left side: Large text input for additional requirements
- Right side: Live mobile chat preview showing the updated prompt
- Displays current agent information and existing system prompt
- Appends additional requirements to the agent's system_prompt on save

### 2. Updated AgentListPage.tsx
- Added `useNavigate` hook import
- Modified `handleTune` method to navigate to `/agents/:id/tune` instead of the detail page
- Added `navigate` to component state

### 3. Updated Router Configuration
Modified `frontend/src/router.tsx`:
- Added import for `AgentTunePage`
- Added new route: `/agents/:id/tune`

### 4. Added Translations

#### Chinese (zh.json)
Added `agents.tune` section with:
- title: "调优数字人"
- description: "为数字人添加额外的要求和指令"
- agentInfo: "数字人信息"
- currentPrompt: "当前系统提示词"
- additionalRequirements: "附加要求"
- requirementsDescription: Instructions for users
- requirementsPlaceholder: Example requirements
- preview: "实时预览"
- saveChanges: "保存调优"
- success/error messages

#### English (en.json)
Added corresponding English translations for all tune-related keys.

## User Flow

1. User navigates to Agents list page
2. Clicks "Tune" button on an employed agent
3. Redirected to `/agents/:id/tune`
4. Views current agent info and system prompt
5. Enters additional requirements in the large text area
6. Sees live preview on the right side with updated prompt
7. Clicks "Save Tuning" to append requirements to system_prompt
8. Redirected back to agents list on success

## Technical Details

- Uses existing `agentService.updateAgent()` API
- Appends additional requirements with double newline separator: `${original_prompt}\n\n${additional_requirements}`
- Preview updates in real-time as user types
- Validates that requirements are not empty before saving
- Shows success/error alerts with proper i18n support

## Files Modified
- `frontend/src/pages/AgentListPage.tsx`
- `frontend/src/router.tsx`
- `frontend/src/i18n/locales/zh.json`
- `frontend/src/i18n/locales/en.json`

## Files Created
- `frontend/src/pages/AgentTunePage.tsx`
