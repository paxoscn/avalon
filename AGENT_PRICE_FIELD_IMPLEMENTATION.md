# Agent Price Field Implementation

## Summary
Added price editing functionality to the Agent Detail Page, allowing users to set a price per 1000 tokens for agents.

## Changes Made

### 1. Type Definitions (`frontend/src/types/index.ts`)
- Added `price?: number` field to the `Agent` interface

### 2. Service Layer (`frontend/src/services/agent.service.ts`)
- Added `price?: number` to `CreateAgentRequest` interface
- Added `price?: number` to `UpdateAgentRequest` interface

### 3. UI Component (`frontend/src/pages/AgentDetailPage.tsx`)
- Added `price: ''` to the form state
- Added price input field in the Basic Information section
- Configured input with:
  - Type: number
  - Step: 0.0001 (supports up to 4 decimal places)
  - Min: 0 (prevents negative values)
- Added price parsing logic in form submission for both create and update operations
- Price is converted from string to float before sending to API

### 4. Internationalization
- **English** (`frontend/src/i18n/locales/en.json`):
  - `agents.detail.price`: "Price per 1K Tokens"
  - `agents.detail.pricePlaceholder`: "0.0000"
  - `agents.detail.priceDescription`: "Price charged per 1000 tokens (up to 4 decimal places)"

- **Chinese** (`frontend/src/i18n/locales/zh.json`):
  - `agents.detail.price`: "每千Token价格"
  - `agents.detail.pricePlaceholder`: "0.0000"
  - `agents.detail.priceDescription`: "每1000个Token收取的价格（最多4位小数）"

## Backend Compatibility
The implementation aligns with the existing backend:
- Backend stores price as `Decimal` with precision (10, 4)
- Backend validates:
  - Non-negative values
  - Maximum 4 decimal places
- Field is optional (nullable)

## UI Location
The price field is positioned in the Basic Information card, right after the Agent Name field and before the Avatar section.

## Validation
- Frontend: HTML5 number input with min="0" and step="0.0001"
- Backend: Validates non-negative values and decimal precision
