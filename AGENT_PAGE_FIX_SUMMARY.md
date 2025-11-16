# Agent Detail Page 修复总结

## 问题描述

AgentDetailPage.tsx 混淆了模型配置（LLM Config）和知识库（Vector Config），需要：
1. 添加大模型配置的单选功能
2. 修正知识库逻辑，使用正确的 Vector Config API

## 修改内容

### 1. 前端页面 (frontend/src/pages/AgentDetailPage.tsx)

#### 新增状态
- 添加 `availableLLMConfigs` 状态用于存储可用的大模型配置
- 在 `formData` 中添加 `llmConfigId` 字段

#### 修改资源加载
```typescript
const loadResources = async () => {
  const [llmConfigs, vectorConfigs, tools, flowsResponse] = await Promise.all([
    llmService.listConfigs(),           // 大模型配置
    vectorService.listConfigs(),        // 知识库配置（向量数据库）
    mcpService.listTools(),
    flowService.getFlows(),
  ]);
  setAvailableLLMConfigs(llmConfigs);
  setAvailableKnowledgeBases(vectorConfigs);  // 现在使用正确的 Vector Config
  // ...
};
```

#### 新增 UI 组件
在"基本信息"和"知识库"之间添加了"大模型配置"卡片：
- 使用单选按钮（radio）选择模型
- 显示模型的 provider 和 model_name
- 只能选择一个模型配置

#### 修改知识库 UI
- 知识库现在显示 Vector Config 的信息
- 显示 provider 信息（pinecone、weaviate 等）
- 支持多选（checkbox）

#### 更新表单提交
- 创建和更新 Agent 时包含 `llm_config_id` 字段
- 使用类型断言处理 TypeScript 类型问题

### 2. 翻译文件

#### 中文翻译 (frontend/src/i18n/locales/zh.json)
```json
{
  "agents": {
    "detail": {
      "llmModel": "大模型配置",
      "llmModelDescription": "选择数字人使用的大语言模型",
      "noLLMConfigs": "暂无可用模型配置"
    }
  }
}
```

#### 英文翻译 (frontend/src/i18n/locales/en.json)
```json
{
  "agents": {
    "detail": {
      "llmModel": "LLM Model Configuration",
      "llmModelDescription": "Select the large language model for the agent to use",
      "noLLMConfigs": "No LLM configurations available"
    }
  }
}
```

## UI 布局顺序

1. **基本信息** - 名称、头像、问候语、系统提示词、附加设置
2. **预设问题** - 最多 3 个快速问题
3. **大模型配置** ⭐ 新增 - 单选模型
4. **知识库** ✅ 修正 - 多选向量数据库
5. **MCP 工具** - 多选工具
6. **工作流** - 多选流程

## 数据结构

### Agent 数据结构（扩展）
```typescript
interface Agent {
  // ... 其他字段
  llm_config_id?: string;        // 新增：大模型配置 ID
  knowledge_base_ids: string[];  // 知识库（Vector Config）ID 列表
  mcp_tool_ids: string[];
  flow_ids: string[];
}
```

### 表单数据
```typescript
{
  llmConfigId: string;              // 单选：大模型配置
  knowledgeBaseIds: string[];       // 多选：知识库
  mcpToolIds: string[];             // 多选：MCP 工具
  flowIds: string[];                // 多选：工作流
}
```

## 关键区别

| 项目 | 大模型配置 (LLM Config) | 知识库 (Vector Config) |
|------|------------------------|----------------------|
| 用途 | 选择对话使用的语言模型 | 选择知识检索的向量数据库 |
| 选择方式 | 单选（radio） | 多选（checkbox） |
| API | `llmService.listConfigs()` | `vectorService.listConfigs()` |
| 显示信息 | provider + model_name | provider |
| 示例 | OpenAI - gpt-4 | Pinecone |

## 测试建议

1. **创建新 Agent**
   - 选择一个大模型配置
   - 选择一个或多个知识库
   - 验证保存后数据正确

2. **编辑现有 Agent**
   - 验证已选择的模型正确显示
   - 验证已选择的知识库正确显示
   - 修改选择并保存

3. **边界情况**
   - 没有可用的大模型配置
   - 没有可用的知识库
   - 不选择大模型配置（可选）

## 后续工作（可选）

- [ ] 后端 API 需要支持 `llm_config_id` 字段
- [ ] 更新 Agent 类型定义以包含 `llm_config_id`
- [ ] 添加模型配置的详细信息展示
- [ ] 添加知识库的详细信息展示
