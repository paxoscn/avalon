# Avatar Upload Implementation

## Changes Made

### 1. AgentDetailPage.tsx
- Added avatar preview with circular image display
- Added upload button with file input
- Added remove avatar button
- Implemented file validation (image type, max 5MB)
- Integrated with existing file upload service
- Added uploading state management

### 2. Translation Files (zh.json & en.json)
Added new translation keys:
- `agents.detail.avatarDescription`: Description text for avatar field
- `agents.detail.uploadAvatar`: Upload button text
- `agents.detail.uploading`: Uploading state text
- `agents.detail.removeAvatar`: Remove button text
- `agents.errors.invalidImageType`: Error for non-image files
- `agents.errors.imageTooLarge`: Error for files > 5MB
- `agents.errors.uploadFailed`: Generic upload error
- `agents.success.avatarUploaded`: Success message

## Features
1. **Avatar Preview**: Shows circular preview of current avatar or placeholder icon
2. **Upload Button**: Opens file picker for image selection
3. **Remove Button**: Clears the avatar URL
4. **Manual URL Input**: Still allows entering avatar URL manually
5. **Validation**: 
   - Only accepts image files
   - Max file size: 5MB
6. **Error Handling**: Shows user-friendly error messages
7. **Loading States**: Displays "Uploading..." during upload

## API Integration
Uses existing `/files/upload` endpoint via `fileService.uploadFile()`
