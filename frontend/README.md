# Agent Platform Frontend

A modern React + TypeScript frontend for the Agent Platform, featuring an Apple-inspired design with Tailwind CSS.

## Tech Stack

- **React 18+** with TypeScript
- **Vite** for fast development and building
- **Tailwind CSS** for styling
- **React Router v6** for routing
- **Zustand** for state management
- **Axios** for API calls
- **Headless UI** for accessible components
- **Heroicons** for icons

## Project Structure

```
src/
├── components/
│   ├── auth/          # Authentication components
│   ├── common/        # Reusable UI components
│   └── layout/        # Layout components (Header, Sidebar)
├── pages/             # Page components
├── services/          # API services
├── stores/            # Zustand stores
├── hooks/             # Custom React hooks
├── types/             # TypeScript type definitions
└── utils/             # Utility functions
```

## Getting Started

### Prerequisites

- Node.js 20.19+ or 22.12+
- npm or yarn

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

The application will be available at `http://localhost:5173`

### Build

```bash
npm run build
```

### Preview Production Build

```bash
npm run preview
```

## Environment Variables

Create a `.env` file in the frontend directory:

```
VITE_API_BASE_URL=http://localhost:3000/api
```

## Features

### Authentication
- Login with tenant ID, username, and password
- JWT token management
- Automatic token refresh
- Protected routes

### UI Components
- Button (multiple variants and sizes)
- Input (with validation)
- Card
- Table (with loading and empty states)
- Alert (info, success, warning, error)
- Modal
- Loader

### Layout
- Responsive sidebar navigation
- Header with user menu
- Apple-inspired design

## Development Guidelines

### Component Structure
- Use functional components with TypeScript
- Implement proper prop types
- Follow the single responsibility principle
- Keep components small and focused

### Styling
- Use Tailwind CSS utility classes
- Follow the design system defined in `tailwind.config.js`
- Use custom CSS classes defined in `index.css` for common patterns

### State Management
- Use Zustand for global state
- Keep local state in components when appropriate
- Implement proper error handling

### API Integration
- All API calls go through the `services/` directory
- Use the centralized `apiClient` for authenticated requests
- Handle errors consistently

## License

Copyright © 2025 Agent Platform. All rights reserved.
