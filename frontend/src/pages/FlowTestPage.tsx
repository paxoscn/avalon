import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { flowService } from '../services/flow.service';
import { fileService } from '../services/file.service';
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
  const [variableTypes, setVariableTypes] = useState<Record<string, string>>({});
  const [uploadedFiles, setUploadedFiles] = useState<Record<string, File[]>>({});
  const [uploadingFiles, setUploadingFiles] = useState<Record<string, boolean>>({});
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

        const initialVars: Record<string, any> = {};
        const types: Record<string, string> = {};
        // initialVars["query"] = '';
        // Extract variables from flow definition if available
        if (currentVersion.definition?.workflow?.graph?.nodes.find((node: any) => node.node_type === "start").data.variables) {
          currentVersion.definition.workflow.graph.nodes.find((node: any) => node.node_type === "start").data.variables.forEach((variable: any) => {
            initialVars[variable.variable] = variable.type === 'file-list' ? [] : variable.default;
            types[variable.variable] = variable.type || 'string';
          });
        }
        setVariables(initialVars);
        setVariableTypes(types);
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
        input_data: executionVariables,
        session_id: sessionId || undefined,
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

  const handleFileSelect = async (variableName: string, files: FileList | null) => {
    if (!files || files.length === 0) return;

    const fileArray = Array.from(files);
    const currentFiles = uploadedFiles[variableName] || [];
    const updatedFiles = [...currentFiles, ...fileArray];
    
    setUploadedFiles({ ...uploadedFiles, [variableName]: updatedFiles });
    setUploadingFiles({ ...uploadingFiles, [variableName]: true });

    try {
      // Upload files and get URLs
      const uploadPromises = fileArray.map(file => uploadFile(file));
      const urls = await Promise.all(uploadPromises);
      
      const currentUrls = variables[variableName] || [];
      const updatedUrls = [...currentUrls, ...urls];
      
      setVariables({ ...variables, [variableName]: updatedUrls });
    } catch (err: any) {
      setError(`文件上传失败: ${err.message}`);
    } finally {
      setUploadingFiles({ ...uploadingFiles, [variableName]: false });
    }
  };

  const uploadFile = async (file: File): Promise<string> => {
    return await fileService.uploadFile(file);
  };

  const removeFile = (variableName: string, index: number) => {
    const currentFiles = uploadedFiles[variableName] || [];
    const currentUrls = variables[variableName] || [];
    
    const updatedFiles = currentFiles.filter((_: File, i: number) => i !== index);
    const updatedUrls = currentUrls.filter((_: string, i: number) => i !== index);
    
    setUploadedFiles({ ...uploadedFiles, [variableName]: updatedFiles });
    setVariables({ ...variables, [variableName]: updatedUrls });
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
                  {Object.keys(variables).map((key) => {
                    const varType = variableTypes[key] || 'string';
                    
                    if (varType === 'file-list') {
                      const files = uploadedFiles[key] || [];
                      const isUploading = uploadingFiles[key] || false;
                      
                      return (
                        <div key={key}>
                          <label className="block text-sm font-medium text-gray-700 mb-1">
                            {key}
                          </label>
                          
                          <div className="space-y-2">
                            {/* File upload button */}
                            <div className="flex items-center gap-2">
                              <label className="cursor-pointer">
                                <input
                                  type="file"
                                  multiple
                                  onChange={(e) => handleFileSelect(key, e.target.files)}
                                  className="hidden"
                                  disabled={isUploading}
                                />
                                <div className="px-4 py-2 bg-blue-50 text-blue-600 rounded-lg hover:bg-blue-100 transition-colors border border-blue-200 text-sm font-medium">
                                  {isUploading ? '上传中...' : '选择文件'}
                                </div>
                              </label>
                              {files.length > 0 && (
                                <span className="text-sm text-gray-500">
                                  已选择 {files.length} 个文件
                                </span>
                              )}
                            </div>
                            
                            {/* File list */}
                            {files.length > 0 && (
                              <div className="space-y-1">
                                {files.map((file, index) => (
                                  <div
                                    key={index}
                                    className="flex items-center justify-between p-2 bg-gray-50 rounded border border-gray-200"
                                  >
                                    <div className="flex items-center gap-2 flex-1 min-w-0">
                                      <svg
                                        className="w-4 h-4 text-gray-400 flex-shrink-0"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                      >
                                        <path
                                          strokeLinecap="round"
                                          strokeLinejoin="round"
                                          strokeWidth={2}
                                          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                                        />
                                      </svg>
                                      <span className="text-sm text-gray-700 truncate">
                                        {file.name}
                                      </span>
                                      <span className="text-xs text-gray-500 flex-shrink-0">
                                        ({(file.size / 1024).toFixed(1)} KB)
                                      </span>
                                    </div>
                                    <button
                                      type="button"
                                      onClick={() => removeFile(key, index)}
                                      className="ml-2 text-red-500 hover:text-red-700 flex-shrink-0"
                                      disabled={isUploading}
                                    >
                                      <svg
                                        className="w-4 h-4"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                      >
                                        <path
                                          strokeLinecap="round"
                                          strokeLinejoin="round"
                                          strokeWidth={2}
                                          d="M6 18L18 6M6 6l12 12"
                                        />
                                      </svg>
                                    </button>
                                  </div>
                                ))}
                              </div>
                            )}
                          </div>
                        </div>
                      );
                    }
                    
                    return (
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
                    );
                  })}
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
