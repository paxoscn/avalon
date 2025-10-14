import { Card } from '../components/common/Card';
import { useAuthStore } from '../stores/authStore';

export const DashboardPage: React.FC = () => {
  const user = useAuthStore((state) => state.user);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
        <p className="text-gray-600 mt-2">
          Welcome back, {user?.nickname || user?.username}!
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">Total Flows</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">Active Executions</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">MCP Tools</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>

        <Card>
          <div className="text-center">
            <p className="text-sm text-gray-600">Sessions</p>
            <p className="text-3xl font-bold text-gray-900 mt-2">0</p>
          </div>
        </Card>
      </div>

      <Card title="Recent Activity" subtitle="Your latest actions">
        <div className="text-center py-8 text-gray-500">
          No recent activity
        </div>
      </Card>
    </div>
  );
};
