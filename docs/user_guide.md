# Agent Platform User Guide

## Introduction

Welcome to the Agent Platform! This guide will help you understand and use all the features of the platform effectively.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Authentication](#authentication)
3. [Flow Management](#flow-management)
4. [MCP Tools](#mcp-tools)
5. [LLM Configuration](#llm-configuration)
6. [Vector Database Configuration](#vector-database-configuration)
7. [Session Management](#session-management)
8. [Audit and Monitoring](#audit-and-monitoring)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

## Getting Started

### Accessing the Platform

1. Open your web browser and navigate to the platform URL
2. You'll see the login page with the Apple-style interface
3. Enter your credentials to access the dashboard

### Dashboard Overview

The dashboard provides:
- Quick stats on flows, executions, and sessions
- Recent activity feed
- System health status
- Quick action buttons

## Authentication

### Logging In

1. Navigate to the login page
2. Enter your tenant ID, username, and password
3. Click "Sign In"
4. You'll receive a JWT token that's valid for 24 hours

### Token Management

- Tokens are automatically refreshed before expiration
- If your session expires, you'll be redirected to login
- You can manually log out from the user menu

### Multi-Tenant Isolation

- Each tenant has isolated data
- Users can only access resources within their tenant
- Tenant ID is required for all operations

## Flow Management

### Creating a Flow

1. Navigate to **Flows** in the sidebar
2. Click **Create New Flow**
3. Enter flow details:
   - Name: Descriptive name for your flow
   - Description: What the flow does
   - Definition: Flow structure (nodes and edges)
4. Click **Save**

### Importing from Dify DSL

1. Go to **Flows** → **Import**
2. Upload your Dify DSL file or paste DSL content
3. The system will parse and validate the DSL
4. Review the imported flow structure
5. Click **Import** to create the flow

### Executing a Flow

1. Open a flow from the flow list
2. Click **Execute**
3. Provide input variables if required
4. Monitor execution status in real-time
5. View results when execution completes

### Version Management

#### Creating a New Version

1. Open a flow
2. Make your changes
3. Click **Save as New Version**
4. Add a change log description
5. The new version becomes active

#### Viewing Version History

1. Open a flow
2. Click **Versions** tab
3. See all versions with timestamps and change logs
4. Compare versions side-by-side

#### Rolling Back

1. Go to **Versions** tab
2. Select the version you want to restore
3. Click **Rollback to This Version**
4. Confirm the action
5. The selected version becomes active

## MCP Tools

### What are MCP Tools?

MCP (Model Context Protocol) tools allow you to integrate external HTTP APIs as callable tools within your flows.

### Configuring an MCP Tool

1. Navigate to **Tools** → **MCP Tools**
2. Click **Add New Tool**
3. Fill in the configuration:
   ```json
   {
     "name": "weather_api",
     "description": "Get weather information",
     "endpoint": "https://api.weather.com/v1/current",
     "method": "GET",
     "headers": {
       "Authorization": "Bearer YOUR_API_KEY"
     },
     "parameters": [
       {
         "name": "city",
         "type": "string",
         "required": true,
         "description": "City name"
       }
     ]
   }
   ```
4. Click **Test Connection** to verify
5. Click **Save**

### Testing an MCP Tool

1. Open the tool configuration
2. Click **Test** tab
3. Provide test parameters
4. Click **Run Test**
5. View the response and execution time

### Using MCP Tools in Flows

MCP tools can be called from flow nodes:

```json
{
  "type": "mcp_tool",
  "tool_id": "weather_api",
  "parameters": {
    "city": "{{user_input.city}}"
  }
}
```

## LLM Configuration

### Supported Providers

- OpenAI (GPT-3.5, GPT-4)
- Anthropic Claude
- Local LLM servers

### Adding an LLM Configuration

1. Go to **Configuration** → **LLM**
2. Click **Add Configuration**
3. Select provider
4. Enter configuration:

#### OpenAI Example
```json
{
  "provider": "openai",
  "api_key": "sk-...",
  "model": "gpt-4",
  "temperature": 0.7,
  "max_tokens": 2000
}
```

#### Claude Example
```json
{
  "provider": "claude",
  "api_key": "sk-ant-...",
  "model": "claude-3-opus",
  "temperature": 0.7,
  "max_tokens": 4000
}
```

4. Click **Test Connection**
5. Set as default if desired
6. Click **Save**

### Testing LLM Configuration

1. Open the configuration
2. Go to **Test** tab
3. Enter a test prompt
4. Click **Send**
5. View the response and latency

## Vector Database Configuration

### Supported Vector Databases

- Pinecone
- Weaviate
- ChromaDB
- Qdrant
- Milvus

### Adding a Vector Database

1. Navigate to **Configuration** → **Vector Databases**
2. Click **Add Configuration**
3. Select provider
4. Enter connection details:

#### Pinecone Example
```json
{
  "provider": "pinecone",
  "api_key": "your-api-key",
  "environment": "us-west1-gcp",
  "index_name": "agent-knowledge"
}
```

#### ChromaDB Example
```json
{
  "provider": "chromadb",
  "host": "localhost",
  "port": 8000,
  "collection_name": "documents"
}
```

4. Click **Test Connection**
5. Click **Save**

### Managing Vector Data

#### Uploading Documents

1. Open vector configuration
2. Go to **Data** tab
3. Click **Upload Documents**
4. Select files or paste text
5. Documents are automatically embedded and stored

#### Searching

1. Go to **Search** tab
2. Enter search query
3. Adjust parameters (top_k, filters)
4. View results with similarity scores

## Session Management

### Creating a Session

Sessions are automatically created when a user starts a conversation:

```bash
POST /api/sessions
{
  "title": "Customer Support Chat"
}
```

### Viewing Session History

1. Navigate to **Sessions**
2. Filter by date, user, or status
3. Click on a session to view details
4. See complete conversation history

### Session Context

Sessions maintain context across messages:
- Previous messages
- User preferences
- Conversation state
- Metadata

## Audit and Monitoring

### Viewing Audit Logs

1. Go to **Audit** → **Logs**
2. Filter by:
   - Date range
   - User
   - Action type
   - Resource type
3. Click on a log entry for details

### Execution History

1. Navigate to **Audit** → **Executions**
2. View all flow executions
3. Filter by status, date, or flow
4. Click on an execution to see:
   - Input/output data
   - Execution steps
   - Performance metrics
   - Error details (if failed)

### Statistics and Analytics

1. Go to **Audit** → **Statistics**
2. View metrics:
   - Total executions
   - Success rate
   - Average execution time
   - Most used flows
   - Error trends

### Exporting Audit Data

1. Select date range and filters
2. Click **Export**
3. Choose format (CSV or JSON)
4. Download the file

## Best Practices

### Flow Design

1. **Keep flows modular**: Break complex flows into smaller, reusable components
2. **Use descriptive names**: Name nodes and variables clearly
3. **Add error handling**: Include error handling nodes for robustness
4. **Test incrementally**: Test each node before building the complete flow
5. **Version control**: Use meaningful change logs when creating versions

### Performance Optimization

1. **Cache frequently used data**: Use session context to avoid redundant API calls
2. **Batch operations**: Group similar operations together
3. **Limit LLM token usage**: Use appropriate max_tokens settings
4. **Index vector data**: Ensure vector databases are properly indexed
5. **Monitor execution times**: Use audit logs to identify slow operations

### Security

1. **Rotate API keys regularly**: Update LLM and vector DB credentials periodically
2. **Use strong passwords**: Follow password best practices
3. **Review audit logs**: Regularly check for suspicious activity
4. **Limit permissions**: Grant minimum necessary permissions
5. **Secure MCP endpoints**: Use authentication for external APIs

### Data Management

1. **Regular backups**: Ensure database backups are running
2. **Clean old sessions**: Archive or delete old session data
3. **Monitor storage**: Keep an eye on database and vector storage usage
4. **Optimize queries**: Use filters to limit data retrieval
5. **Document flows**: Add descriptions and comments to flows

## Troubleshooting

### Common Issues

#### Flow Execution Fails

**Symptoms**: Flow shows "Failed" status

**Solutions**:
1. Check execution details for error message
2. Verify all required inputs are provided
3. Test individual nodes separately
4. Check LLM/vector DB configurations
5. Review audit logs for detailed error info

#### LLM Not Responding

**Symptoms**: Timeout or no response from LLM

**Solutions**:
1. Test LLM configuration connection
2. Verify API key is valid
3. Check rate limits on provider
4. Try reducing max_tokens
5. Switch to alternative LLM configuration

#### Vector Search Returns No Results

**Symptoms**: Empty search results

**Solutions**:
1. Verify documents are uploaded
2. Check collection/index name
3. Adjust similarity threshold
4. Test with simpler queries
5. Verify vector DB connection

#### Session Context Lost

**Symptoms**: Bot doesn't remember previous messages

**Solutions**:
1. Check session ID is being passed
2. Verify session hasn't expired
3. Review session configuration
4. Check Redis connection
5. Look for errors in audit logs

### Getting Help

#### Self-Service Resources

1. **Documentation**: Check this guide and API docs
2. **Audit Logs**: Review logs for error details
3. **Health Check**: Visit `/health/detailed` endpoint
4. **Test Connections**: Use built-in connection testers

#### Support Channels

1. **GitHub Issues**: Report bugs and feature requests
2. **Community Forum**: Ask questions and share solutions
3. **Email Support**: support@example.com
4. **Documentation**: docs.example.com

### Error Messages

#### "Authentication Failed"
- Verify credentials are correct
- Check if token has expired
- Ensure tenant ID is correct

#### "Resource Not Found"
- Verify resource ID is correct
- Check if resource was deleted
- Ensure you have access permissions

#### "Rate Limit Exceeded"
- Wait before retrying
- Check rate limit settings
- Consider upgrading plan

#### "Database Connection Failed"
- Check database is running
- Verify connection string
- Contact system administrator

## Keyboard Shortcuts

- `Ctrl/Cmd + K`: Quick search
- `Ctrl/Cmd + N`: New flow
- `Ctrl/Cmd + S`: Save current item
- `Ctrl/Cmd + E`: Execute flow
- `Esc`: Close modal/dialog

## Tips and Tricks

1. **Use templates**: Start with flow templates for common patterns
2. **Duplicate flows**: Clone existing flows to save time
3. **Bulk operations**: Select multiple items for batch actions
4. **Keyboard navigation**: Use shortcuts for faster workflow
5. **Save frequently**: Auto-save is enabled, but manual saves are instant
6. **Test in sandbox**: Use test mode before production deployment
7. **Monitor metrics**: Keep an eye on performance dashboards
8. **Set up alerts**: Configure notifications for important events

## Glossary

- **Flow**: A sequence of operations that defines agent behavior
- **Node**: A single operation within a flow
- **Execution**: A single run of a flow with specific inputs
- **Session**: A conversation context that persists across messages
- **MCP Tool**: An external API integrated as a callable tool
- **Vector Database**: Storage for embeddings and semantic search
- **Tenant**: An isolated workspace for an organization
- **Audit Log**: Record of all actions performed in the system

## Appendix

### API Quick Reference

```bash
# Authentication
POST /api/auth/login
POST /api/auth/refresh
POST /api/auth/logout

# Flows
GET /api/flows
POST /api/flows
GET /api/flows/{id}
PUT /api/flows/{id}
DELETE /api/flows/{id}
POST /api/flows/{id}/execute

# MCP Tools
GET /api/mcp/tools
POST /api/mcp/tools
POST /api/mcp/tools/{id}/call

# Sessions
GET /api/sessions
POST /api/sessions
GET /api/sessions/{id}
POST /api/sessions/{id}/messages

# Audit
GET /api/audit
GET /api/audit/statistics
GET /api/executions
GET /api/executions/{id}
```

### Configuration Examples

See the [API Documentation](api_documentation.md) for detailed configuration examples and schemas.

---

**Last Updated**: 2024-01-01  
**Version**: 1.0.0
