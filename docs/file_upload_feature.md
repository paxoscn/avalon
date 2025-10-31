# File Upload Feature for Flow Test Page

## Overview
The Flow Test Page now supports file uploads for variables with type `file-list`. Users can upload multiple files, and the system will pass the file download URLs to the flow execution.

## Features

### 1. File Upload UI
- Variables with type `file-list` display a file upload button
- Users can select multiple files at once
- Each file is displayed with its name and size
- Files can be removed individually before execution

### 2. File Management
- Files are uploaded immediately upon selection
- Upload progress is indicated with a loading state
- File URLs are stored in the variable value as an array
- The URLs are passed to the flow execution

### 3. User Experience
- Clean, intuitive interface with file list display
- Visual feedback during upload
- Error handling for failed uploads
- Seamless integration with existing form/JSON modes

## Implementation Details

### Frontend Changes

#### FlowTestPage.tsx
Added state management for:
- `variableTypes`: Tracks the type of each variable
- `uploadedFiles`: Stores File objects for display
- `uploadingFiles`: Tracks upload status per variable

Key functions:
- `handleFileSelect`: Handles file selection and triggers upload
- `uploadFile`: Uploads a single file and returns its URL
- `removeFile`: Removes a file from the list

#### file.service.ts
New service for file upload operations:
- `uploadFile(file: File)`: Uploads a single file
- `uploadFiles(files: File[])`: Uploads multiple files in parallel

### Backend Requirements

You need to implement a file upload endpoint:

```
POST /files/upload
Content-Type: multipart/form-data

Request Body:
- file: File (multipart)

Response:
{
  "url": "https://your-storage.com/files/filename.ext",
  "filename": "filename.ext",
  "size": 12345,
  "contentType": "image/png"
}
```

### Variable Type Detection

The system detects `file-list` type variables from the flow definition:

```json
{
  "workflow": {
    "graph": {
      "nodes": [
        {
          "node_type": "start",
          "data": {
            "variables": [
              {
                "variable": "documents",
                "type": "file-list",
                "default": []
              }
            ]
          }
        }
      ]
    }
  }
}
```

## Usage Example

1. Create a flow with a start node that has a `file-list` variable
2. Navigate to the flow test page
3. Click "选择文件" (Select Files) button
4. Choose one or more files
5. Files are uploaded automatically
6. Click "执行Flow" to run the flow with the file URLs

## Future Enhancements

- File type restrictions (e.g., only images, PDFs)
- File size limits
- Drag-and-drop support
- Upload progress bars
- Preview for image files
- Batch upload optimization
