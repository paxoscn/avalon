# Agent Interview Feature Implementation

## Overview
Implemented a new interview page for agents that allows users to evaluate and employ agents from the "Visible" tab.

## Changes Made

### 1. Created AgentInterviewPage Component
**File**: `frontend/src/pages/AgentInterviewPage.tsx`

Features:
- **Left Panel - Evaluation Form**:
  - Agent information display (avatar, name, creation date)
  - 5 scoring categories with sliders (1-10 scale):
    - Professionalism (专业性)
    - Communication (沟通能力)
    - Knowledge (知识储备)
    - Responsiveness (响应速度)
    - Overall (综合评价)
  - Average score calculation
  - Interview feedback textarea
  - Employ button with confirmation

- **Right Panel - Live Preview**:
  - Mobile chat preview using MobileChatPreview component
  - Real-time interaction with the agent
  - Same preview component used in agent edit page

### 2. Updated AgentListPage
**File**: `frontend/src/pages/AgentListPage.tsx`

- Modified `handleInterview` method to navigate to `/agents/:id/interview` route
- Changed from `window.location.href` to `navigate()` for proper SPA routing

### 3. Added Route Configuration
**File**: `frontend/src/router.tsx`

- Added new route: `/agents/:id/interview` → `<AgentInterviewPage />`
- Imported AgentInterviewPage component

### 4. Translation Updates

**Chinese** (`frontend/src/i18n/locales/zh.json`):
```json
"interview": {
  "title": "面试数字人",
  "description": "评估数字人的能力并决定是否雇佣",
  "agentInfo": "数字人信息",
  "createdAt": "创建时间",
  "evaluationScores": "评估打分",
  "professionalism": "专业性",
  "communication": "沟通能力",
  "knowledge": "知识储备",
  "responsiveness": "响应速度",
  "overall": "综合评价",
  "averageScore": "平均分",
  "feedback": "面试反馈",
  "feedbackPlaceholder": "请输入您对这个数字人的面试评价和建议...",
  "feedbackDescription": "记录您在面试过程中的观察和评价",
  "employDecision": "雇佣决策",
  "employDecisionDescription": "如果您对这个数字人满意，可以点击雇佣按钮",
  "employing": "雇佣中...",
  "employSuccess": "数字人雇佣成功！",
  "preview": "实时预览",
  "previewDescription": "与数字人进行对话测试"
}
```

**English** (`frontend/src/i18n/locales/en.json`):
- Added corresponding English translations for all interview-related keys

## User Flow

1. User navigates to Agents page
2. Switches to "Visible" tab to see available agents
3. Clicks "Interview" button on an agent card
4. Navigates to interview page with:
   - Left side: Evaluation form with scoring sliders and feedback
   - Right side: Live chat preview to test the agent
5. User interacts with agent in preview panel
6. User fills out evaluation scores (1-10 for each category)
7. User writes feedback notes
8. User clicks "Employ" button to hire the agent
9. Success message shown and redirects back to agents list

## Technical Details

- Uses existing `agentService.employAgent()` API method
- Reuses `MobileChatPreview` component for consistency
- Follows same layout pattern as AgentDetailPage and AgentTunePage
- Responsive design with sticky right panel
- Error handling with Alert component
- Loading states with Loader component
- i18n support for both Chinese and English

## UI Components Used

- `Button` - Action buttons
- `Card` - Content containers
- `Loader` - Loading indicator
- `Alert` - Error/success messages
- `MobileChatPreview` - Agent chat interface preview

## Styling

- Consistent with existing agent pages
- Range sliders with blue accent color
- Gradient avatar fallback
- Sticky preview panel (top-6, self-start)
- 700px preview height
- Responsive gap-6 layout
