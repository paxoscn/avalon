import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import yaml from 'js-yaml';
import { flowService } from '../services/flow.service';
import type { Flow, ValidationResult } from '../types';
import { Button, Card, Alert, Input } from '../components/common';

export const FlowImportPage = () => {
  const navigate = useNavigate();
  const [flowName, setFlowName] = useState('');
  const [dslContent, setDslContent] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [importing, setImporting] = useState(false);
  const [validating, setValidating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [importedFlow, setImportedFlow] = useState<Flow | null>(null);
  const [previewMode, setPreviewMode] = useState(false);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFile = e.target.files?.[0];
    if (selectedFile) {
      setFile(selectedFile);
      const reader = new FileReader();
      reader.onload = (event) => {
        const content = event.target?.result as string;
        setDslContent(content);
        setValidation(null);
        setImportedFlow(null);
      };
      reader.readAsText(selectedFile);
    }
  };

  const handleValidate = async () => {
    if (!dslContent.trim()) {
      setError('Please provide DSL content');
      return;
    }

    if (!flowName.trim()) {
      setError('Please provide a flow name');
      return;
    }

    try {
      setValidating(true);
      setError(null);
      setValidation(null);

      // Parse DSL to validate YAML format
      let parsedDsl: any;
      try {
        parsedDsl = yaml.load(dslContent);
      } catch (e: any) {
        setError(`Invalid YAML format: ${e.message}`);
        return;
      }

      // Perform basic validation
      const errors: string[] = [];
      const warnings: string[] = [];

      if (!parsedDsl || typeof parsedDsl !== 'object') {
        errors.push('DSL must be a valid YAML object');
      } else {
        if (!parsedDsl.workflow.graph.nodes || !Array.isArray(parsedDsl.workflow.graph.nodes)) {
          errors.push('DSL must contain a "nodes" array');
        }

        if (parsedDsl.workflow.graph.nodes && parsedDsl.workflow.graph.nodes.length === 0) {
          warnings.push('Flow has no nodes defined');
        }

        if (!parsedDsl.workflow.graph.edges || !Array.isArray(parsedDsl.workflow.graph.edges)) {
          warnings.push('DSL should contain an "edges" array for node connections');
        }
      }

      const validationResult: ValidationResult = {
        is_valid: errors.length === 0,
        errors: errors.length > 0 ? errors : undefined,
        warnings: warnings.length > 0 ? warnings : undefined,
      };

      setValidation(validationResult);
      setPreviewMode(true);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to validate DSL');
    } finally {
      setValidating(false);
    }
  };

  const handleImport = async () => {
    if (!dslContent.trim() || !flowName.trim()) {
      setError('Please provide both flow name and DSL content');
      return;
    }

    try {
      setImporting(true);
      setError(null);

      const result = await flowService.importDsl({
        dsl: JSON.stringify(((obj: any) => ({ ...obj, workflow: { ...obj.workflow, graph: { ...obj.workflow.graph, nodes: obj.workflow.graph.nodes.map((node: any) => { node.node_type = node.data.type; return node }) } } }))(yaml.load(dslContent)), null, 2),
        name: flowName,
      });

      setImportedFlow(result.flow);
      setValidation(result.validation);

      if (result.validation.is_valid) {
        setTimeout(() => {
          navigate(`/flows/${result.flow.id}`);
        }, 2000);
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to import DSL');
    } finally {
      setImporting(false);
    }
  };

  const handleReset = () => {
    setFlowName('');
    setDslContent('');
    setFile(null);
    setValidation(null);
    setImportedFlow(null);
    setPreviewMode(false);
    setError(null);
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <Button variant="secondary" size="sm" onClick={() => navigate('/flows')}>
            ← Back to Flows
          </Button>
          <h1 className="text-2xl font-semibold text-gray-900 mt-2">Import Dify DSL</h1>
          <p className="text-gray-600 mt-1">
            Upload or paste a Dify DSL file to create a new flow
          </p>
        </div>
        {(validation || importedFlow) && (
          <Button variant="secondary" onClick={handleReset}>
            Reset
          </Button>
        )}
      </div>

      {error && (
        <Alert type="error" onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {importedFlow && validation?.is_valid && (
        <Alert type="success">
          Flow imported successfully! Redirecting to flow details...
        </Alert>
      )}

      {!previewMode ? (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card>
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Flow Information</h2>
            <div className="space-y-4">
              <Input
                label="Flow Name"
                value={flowName}
                onChange={(e) => setFlowName(e.target.value)}
                placeholder="Enter flow name"
                required
              />

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Upload DSL File
                </label>
                <input
                  type="file"
                  accept=".json,.yaml,.yml"
                  onChange={handleFileChange}
                  className="block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-lg file:border-0 file:text-sm file:font-semibold file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100"
                />
                {file && (
                  <p className="mt-2 text-sm text-gray-600">Selected: {file.name}</p>
                )}
              </div>
            </div>
          </Card>

          <Card>
            <h2 className="text-lg font-semibold text-gray-900 mb-4">DSL Content</h2>
            <textarea
              value={dslContent}
              onChange={(e) => setDslContent(e.target.value)}
              className="w-full h-64 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono text-sm"
              placeholder='Paste your Dify DSL here or upload a file...'
            />
            <div className="mt-4 flex gap-3">
              <Button
                onClick={handleValidate}
                disabled={validating || !dslContent.trim() || !flowName.trim()}
                className="flex-1"
              >
                {validating ? 'Validating...' : 'Validate & Preview'}
              </Button>
            </div>
          </Card>
        </div>
      ) : (
        <div className="space-y-6">
          {validation && (
            <Card>
              <h2 className="text-lg font-semibold text-gray-900 mb-4">Validation Results</h2>
              <div className="space-y-3">
                <div className="flex items-center gap-2">
                  <div
                    className={`w-3 h-3 rounded-full ${validation.is_valid ? 'bg-green-500' : 'bg-red-500'
                      }`}
                  />
                  <span className="font-medium">
                    {validation.is_valid ? 'Valid DSL' : 'Invalid DSL'}
                  </span>
                </div>

                {validation.errors && validation.errors.length > 0 && (
                  <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                    <h3 className="text-sm font-semibold text-red-800 mb-2">Errors:</h3>
                    <ul className="list-disc list-inside space-y-1">
                      {validation.errors.map((err, idx) => (
                        <li key={idx} className="text-sm text-red-700">
                          {err}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}

                {validation.warnings && validation.warnings.length > 0 && (
                  <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                    <h3 className="text-sm font-semibold text-yellow-800 mb-2">Warnings:</h3>
                    <ul className="list-disc list-inside space-y-1">
                      {validation.warnings.map((warn, idx) => (
                        <li key={idx} className="text-sm text-yellow-700">
                          {warn}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}

                {validation.is_valid && !validation.warnings && (
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                    <p className="text-sm text-green-700">
                      ✓ DSL is valid and ready to import
                    </p>
                  </div>
                )}
              </div>
            </Card>
          )}

          <Card>
            <h2 className="text-lg font-semibold text-gray-900 mb-4">Flow Preview</h2>
            <div className="space-y-4">
              <div>
                <h3 className="text-sm font-medium text-gray-700 mb-2">Flow Name</h3>
                <p className="text-gray-900">{flowName}</p>
              </div>

              <div>
                <h3 className="text-sm font-medium text-gray-700 mb-2">DSL Structure</h3>
                <pre className="bg-gray-50 p-4 rounded-lg overflow-x-auto text-xs max-h-96">
                  {JSON.stringify(yaml.load(dslContent), null, 2)}
                </pre>
              </div>

              {validation?.is_valid && (
                <div className="pt-4 border-t border-gray-200">
                  <Button
                    onClick={handleImport}
                    disabled={importing || !validation.is_valid}
                    className="w-full"
                  >
                    {importing ? 'Importing...' : 'Import Flow'}
                  </Button>
                </div>
              )}
            </div>
          </Card>
        </div>
      )}

      <Card>
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Import Guidelines</h2>
        <div className="space-y-3 text-sm text-gray-700">
          <div>
            <h3 className="font-medium text-gray-900 mb-1">Supported Format</h3>
            <p>The DSL must be in valid YAML format following the Dify DSL specification.</p>
          </div>
          <div>
            <h3 className="font-medium text-gray-900 mb-1">Required Fields</h3>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>nodes: Array of flow nodes</li>
              <li>edges: Array of connections between nodes (recommended)</li>
            </ul>
          </div>
          <div>
            <h3 className="font-medium text-gray-900 mb-1">Common Issues</h3>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>Invalid YAML syntax - ensure proper formatting and indentation</li>
              <li>Missing required fields - check the DSL structure</li>
              <li>Unsupported node types - verify node compatibility</li>
            </ul>
          </div>
        </div>
      </Card>
    </div>
  );
};
