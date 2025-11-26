import { createBrowserRouter, Navigate } from 'react-router-dom';
import { ProtectedRoute } from './components/auth/ProtectedRoute';
import { MainLayout } from './components/layout/MainLayout';
import { LoginPage } from './pages/LoginPage';
import { DashboardPage } from './pages/DashboardPage';
import { FlowListPage } from './pages/FlowListPage';
import { FlowDetailPage } from './pages/FlowDetailPage';
import { FlowExecutionPage } from './pages/FlowExecutionPage';
import { FlowVersionsPage } from './pages/FlowVersionsPage';
import { FlowImportPage } from './pages/FlowImportPage';
import { FlowTestPage } from './pages/FlowTestPage';
import { MCPToolListPage } from './pages/MCPToolListPage';
import { MCPToolDetailPage } from './pages/MCPToolDetailPage';
import { MCPToolTestPage } from './pages/MCPToolTestPage';
import { MCPToolVersionsPage } from './pages/MCPToolVersionsPage';
import { LLMConfigListPage } from './pages/LLMConfigListPage';
import { LLMConfigDetailPage } from './pages/LLMConfigDetailPage';
import { LLMConfigTestPage } from './pages/LLMConfigTestPage';
import { VectorConfigListPage } from './pages/VectorConfigListPage';
import { VectorConfigDetailPage } from './pages/VectorConfigDetailPage';
import { VectorConfigTestPage } from './pages/VectorConfigTestPage';
import { AgentListPage } from './pages/AgentListPage';
import { AgentDetailPage } from './pages/AgentDetailPage';
import { AgentTunePage } from './pages/AgentTunePage';
import { AuditLogPage } from './pages/AuditLogPage';
import { AuditLogDetailPage } from './pages/AuditLogDetailPage';
import { ExecutionHistoryPage } from './pages/ExecutionHistoryPage';
import { ExecutionDetailPage } from './pages/ExecutionDetailPage';
import { SessionHistoryPage } from './pages/SessionHistoryPage';
import { SessionDetailPage } from './pages/SessionDetailPage';

export const router = createBrowserRouter([
  {
    path: '/login',
    element: <LoginPage />,
  },
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <MainLayout />
      </ProtectedRoute>
    ),
    children: [
      {
        index: true,
        element: <Navigate to="/dashboard" replace />,
      },
      {
        path: 'dashboard',
        element: <DashboardPage />,
      },
      {
        path: 'flows',
        element: <FlowListPage />,
      },
      {
        path: 'flows/import',
        element: <FlowImportPage />,
      },
      {
        path: 'flows/:id',
        element: <FlowDetailPage />,
      },
      {
        path: 'flows/:id/test',
        element: <FlowTestPage />,
      },
      {
        path: 'flows/:id/versions',
        element: <FlowVersionsPage />,
      },
      {
        path: 'flows/:flowId/executions/:executionId',
        element: <FlowExecutionPage />,
      },
      {
        path: 'mcp/tools',
        element: <MCPToolListPage />,
      },
      {
        path: 'mcp/tools/:id',
        element: <MCPToolDetailPage />,
      },
      {
        path: 'mcp/tools/:id/test',
        element: <MCPToolTestPage />,
      },
      {
        path: 'mcp/tools/:id/versions',
        element: <MCPToolVersionsPage />,
      },
      {
        path: 'config/llm',
        element: <LLMConfigListPage />,
      },
      {
        path: 'config/llm/:id',
        element: <LLMConfigDetailPage />,
      },
      {
        path: 'config/llm/:id/test',
        element: <LLMConfigTestPage />,
      },
      {
        path: 'config/vector',
        element: <VectorConfigListPage />,
      },
      {
        path: 'config/vector/:id',
        element: <VectorConfigDetailPage />,
      },
      {
        path: 'config/vector/:id/test',
        element: <VectorConfigTestPage />,
      },
      {
        path: 'agents',
        element: <AgentListPage />,
      },
      {
        path: 'agents/:id',
        element: <AgentDetailPage />,
      },
      {
        path: 'agents/:id/tune',
        element: <AgentTunePage />,
      },
      {
        path: 'audit/logs',
        element: <AuditLogPage />,
      },
      {
        path: 'audit/logs/:id',
        element: <AuditLogDetailPage />,
      },
      {
        path: 'executions',
        element: <ExecutionHistoryPage />,
      },
      {
        path: 'executions/:id',
        element: <ExecutionDetailPage />,
      },
      {
        path: 'sessions',
        element: <SessionHistoryPage />,
      },
      {
        path: 'sessions/:id',
        element: <SessionDetailPage />,
      },
    ],
  },
  {
    path: '*',
    element: <Navigate to="/" replace />,
  },
]);
