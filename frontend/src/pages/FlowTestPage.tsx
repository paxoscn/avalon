import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { flowService } from '../services/flow.service';
import type { Flow, FlowVersion } from '../types';
import { Button, Card, Input, Loader, Alert } from '../components/common';

export function FlowTestPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const [flow, setFlow] = useState<Flow | null>(null);
  const [version, setVersion] = useState<FlowVersion | null>(null);
  const [loading, setLoading] = useState(true);
  const [executing, setExecuting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [variables, setVariables] = useState<Record<string, any>>({});
  const [sessionId, setSessionId] = useState<string>('');
  const [jsonInput, setJsonInput] = useState<string>('{}');
  const [useJsonMode, setUseJsonMode] = useState(false);
  const [jsonError, setJsonError] = useState<string | null>(null);

  useEffect(() => {
    if (id) {
      loadFlow();
    }
  }, [id]);

  const loadFlow = async () => {
    try {
      setLoading(true);
      setError(null);

      const flowData = await flowService.getFlowById(id!);
      setFlow(flowData);

      const versions = await flowService.getFlowVersions(id!);
      if (versions.length > 0) {
        const currentVersion = versions.find(v => v.version === flowData.current_version) || versions[0];
        setVersion(currentVersion);

        console.log("xxx", currentVersion.definition?.workflow?.graph?.nodes.find((node: any) => node.node_type === "start"));
        // Extract variables from flow definition if available
        if (currentVersion.definition?.workflow?.graph?.nodes.find((node: any) => node.node_type === "start").data.variables) {
          const initialVars: Record<string, any> = {};
          currentVersion.definition.workflow.graph.nodes.find((node: any) => node.node_type === "start").data.variables.forEach((variable: any) => {
            initialVars[variable.variable] = variable.default;
          });
          setVariables(initialVars);
        }
      }
    } catch (err: any) {
      setError(err.response?.data?.error || '加载Flow失败');
    } finally {
      setLoading(false);
    }
  };

  const handleExecute = async () => {
    setExecuting(true);
    setError(null);
    setJsonError(null);

    try {
      let executionVariables = variables;

      if (useJsonMode) {
        try {
          executionVariables = JSON.parse(jsonInput);
        } catch (e) {
          setJsonError('JSON格式错误，请检查输入');
          setExecuting(false);
          return;
        }
      }

      const result = await flowService.executeFlow(id!, {
        variables: executionVariables,
        sessionId: sessionId || undefined,
      });

      // Navigate to execution detail page
      navigate(`/flows/${id}/executions/${result.id}`);
    } catch (err: any) {
      setError(err.response?.data?.error || '执行Flow失败');
    } finally {
      setExecuting(false);
    }
  };

  const handleVariableChange = (name: string, value: any) => {
    setVariables({ ...variables, [name]: value });
  };

  const toggleInputMode = () => {
    if (!useJsonMode) {
      // Switching to JSON mode
      setJsonInput(JSON.stringify(variables, null, 2));
    } else {
      // Switching to form mode
      try {
        const parsed = JSON.parse(jsonInput);
        setVariables(parsed);
        setJsonError(null);
      } catch (e) {
        setJsonError('JSON格式错误，无法切换到表单模式');
        return;
      }
    }
    setUseJsonMode(!useJsonMode);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader size="lg" />
      </div>
    );
  }

  if (!flow) {
    return (
      <Alert type="error">
        Flow未找到
      </Alert>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-gray-900">测试Flow</h1>
          <p className="mt-2 text-sm text-gray-600">
            填写参数并执行Flow以查看结果
          </p>
        </div>
        <Link to={`/flows/${id}`}>
          <Button variant="secondary">返回Flow详情</Button>
        </Link>
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      <Card>
        <h2 className="text-lg font-medium text-gray-900 mb-4">Flow信息</h2>
        <div className="space-y-2 text-sm">
          <div className="flex">
            <span className="font-medium w-32">名称:</span>
            <span className="text-gray-600">{flow.name}</span>
          </div>
          {flow.description && (
            <div className="flex">
              <span className="font-medium w-32">描述:</span>
              <span className="text-gray-600">{flow.description}</span>
            </div>
          )}
          <div className="flex">
            <span className="font-medium w-32">当前版本:</span>
            <span className="text-gray-600">v{flow.current_version}</span>
          </div>
          <div className="flex">
            <span className="font-medium w-32">状态:</span>
            <span
              className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                flow.status === 'Active'
                  ? 'bg-green-100 text-green-800'
                  : flow.status === 'Draft'
                  ? 'bg-yellow-100 text-yellow-800'
                  : 'bg-gray-100 text-gray-800'
              }`}
            >
              {flow.status}
            </span>
          </div>
        </div>
      </Card>

      <Card>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-medium text-gray-900">执行参数</h2>
          <Button
            variant="secondary"
            size="sm"
            onClick={toggleInputMode}
          >
            {useJsonMode ? '切换到表单模式' : '切换到JSON模式'}
          </Button>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Session ID (可选)
            </label>
            <Input
              type="text"
              value={sessionId}
              onChange={(e) => setSessionId(e.target.value)}
              placeholder="留空则自动创建新会话"
            />
          </div>

          {useJsonMode ? (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                变量 (JSON格式)
              </label>
              {jsonError && (
                <div className="mb-2 p-2 bg-red-50 border border-red-200 rounded text-sm text-red-600">
                  {jsonError}
                </div>
              )}
              <textarea
                value={jsonInput}
                onChange={(e) => {
                  setJsonInput(e.target.value);
                  setJsonError(null);
                }}
                rows={10}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
                placeholder='{"key": "value"}'
              />
            </div>
          ) : (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                变量
              </label>
              {Object.keys(variables).length === 0 ? (
                <div className="text-sm text-gray-500 p-4 bg-gray-50 rounded-lg">
                  此Flow没有定义变量，或者您可以切换到JSON模式手动输入变量
                </div>
              ) : (
                <div className="space-y-3">
                  {Object.keys(variables).map((key) => (
                    <div key={key}>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        {key}
                      </label>
                      <Input
                        type="text"
                        value={variables[key] || ''}
                        onChange={(e) => handleVariableChange(key, e.target.value)}
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        <div className="mt-6">
          <Button
            onClick={handleExecute}
            disabled={executing || flow.status !== 'Active'}
            className="w-full sm:w-auto"
          >
            {executing ? '执行中...' : '执行Flow'}
          </Button>
          {flow.status !== 'Active' && (
            <p className="mt-2 text-sm text-amber-600">
              Flow必须处于Active状态才能执行
            </p>
          )}
        </div>
      </Card>

      {version && version.definition && (
        <Card>
          <h2 className="text-lg font-medium text-gray-900 mb-4">Flow定义</h2>
          <pre className="p-4 bg-gray-50 rounded-lg overflow-x-auto text-sm">
            {JSON.stringify(version.definition, null, 2)}
          </pre>
        </Card>
      )}
    </div>
  );
}
