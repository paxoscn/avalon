# MCP工具编辑页面属性验证

## 概述
本文档验证MCP工具编辑页面是否支持编辑所有HTTPToolConfig属性。

## 后端支持的属性

### HTTPToolConfig (src/domain/value_objects/tool_config.rs)
```rust
pub struct HTTPToolConfig {
    pub endpoint: String,              // ✅ 支持编辑
    pub method: HttpMethod,            // ✅ 支持编辑
    pub headers: HashMap<String, String>, // ✅ 支持编辑
    pub parameters: Vec<ParameterSchema>, // ✅ 支持编辑
    pub timeout_seconds: Option<u64>,  // ✅ 支持编辑
    pub retry_count: Option<u32>,      // ✅ 支持编辑
    pub response_template: Option<String>, // ✅ 支持编辑
}
```

### ParameterSchema
```rust
pub struct ParameterSchema {
    pub name: String,                  // ✅ 支持编辑
    pub parameter_type: ParameterType, // ✅ 支持编辑
    pub description: Option<String>,   // ✅ 支持编辑
    pub required: bool,                // ✅ 支持编辑
    pub default_value: Option<serde_json::Value>, // ✅ 支持编辑
    pub enum_values: Option<Vec<serde_json::Value>>, // ✅ 支持编辑
    pub position: ParameterPosition,   // ✅ 支持编辑
}
```

## 前端编辑页面支持

### 基本信息
- ✅ **name**: 工具名称
- ✅ **description**: 工具描述

### HTTP配置
- ✅ **endpoint**: 端点URL（支持路径参数占位符 {paramName}）
- ✅ **method**: HTTP方法（GET/POST/PUT/DELETE/PATCH）
- ✅ **timeout_seconds**: 超时时间（1-300秒）
- ✅ **retry_count**: 重试次数（0-10次）
- ✅ **response_template**: 响应模板（Handlebars语法）

### Headers
- ✅ 动态添加/删除HTTP头
- ✅ 键值对编辑

### Parameters
每个参数支持以下属性：
- ✅ **name**: 参数名称
- ✅ **parameter_type**: 参数类型（String/Number/Boolean/Object/Array）
- ✅ **position**: 参数位置（body/header/path）
- ✅ **description**: 参数描述
- ✅ **required**: 是否必需
- ✅ **default_value**: 默认值（JSON格式）
- ✅ **enum_values**: 枚举值（JSON数组格式）

### 版本管理
- ✅ **changeLog**: 变更日志（更新时）

## UI改进

### 新增字段说明
1. **Timeout (seconds)**: 输入框，范围1-300秒，默认30秒
2. **Retry Count**: 输入框，范围0-10次，默认3次
3. **Response Template**: 多行文本框，支持Handlebars模板语法
4. **Parameter Position**: 下拉选择（Body/Header/Path）
5. **Default Value**: JSON格式输入
6. **Enum Values**: JSON数组格式输入

### 验证规则
- Endpoint URL必须是有效的URL格式
- 路径参数必须在endpoint中有对应的占位符
- Header参数名称只能包含字母、数字和连字符
- Timeout范围：1-300秒
- Retry Count范围：0-10次
- Default Value和Enum Values必须是有效的JSON

## 测试场景

### 场景1：创建带路径参数的工具
```json
{
  "name": "get-user",
  "endpoint": "https://api.example.com/users/{userId}",
  "method": "GET",
  "parameters": [
    {
      "name": "userId",
      "parameter_type": "String",
      "position": "path",
      "required": true
    }
  ]
}
```

### 场景2：创建带Header参数的工具
```json
{
  "name": "auth-api",
  "endpoint": "https://api.example.com/data",
  "method": "GET",
  "parameters": [
    {
      "name": "Authorization",
      "parameter_type": "String",
      "position": "header",
      "required": true
    }
  ]
}
```

### 场景3：创建带枚举值的工具
```json
{
  "name": "update-status",
  "endpoint": "https://api.example.com/status",
  "method": "POST",
  "parameters": [
    {
      "name": "status",
      "parameter_type": "String",
      "position": "body",
      "required": true,
      "enum_values": ["active", "inactive", "pending"]
    }
  ]
}
```

### 场景4：创建带响应模板的工具
```json
{
  "name": "weather-api",
  "endpoint": "https://api.weather.com/current",
  "method": "GET",
  "timeout_seconds": 10,
  "retry_count": 2,
  "response_template": "Temperature: {{ data.temp }}°C, Condition: {{ data.condition }}"
}
```

## 文件修改清单

### 前端文件
1. ✅ `frontend/src/types/index.ts` - 更新类型定义
2. ✅ `frontend/src/services/mcp.service.ts` - 更新服务接口
3. ✅ `frontend/src/pages/MCPToolDetailPage.tsx` - 添加编辑字段
4. ✅ `frontend/src/components/common/Input.tsx` - 添加helpText支持

### 后端文件
- 无需修改，后端已完全支持所有属性

## 验证结果

✅ **所有HTTPToolConfig属性都可以在前端编辑页面中编辑**

- 基本配置：endpoint, method ✅
- 高级配置：timeout_seconds, retry_count, response_template ✅
- Headers：动态添加/删除 ✅
- Parameters：所有7个属性都支持编辑 ✅
  - name, parameter_type, description, required ✅
  - position, default_value, enum_values ✅

## 下一步建议

1. 添加实时验证提示
   - URL格式验证
   - 路径参数一致性检查
   - JSON格式验证

2. 改进用户体验
   - 添加参数位置的图标提示
   - 提供常用模板示例
   - 添加响应模板语法高亮

3. 增强文档
   - 添加Handlebars模板语法指南
   - 提供更多参数配置示例
   - 创建最佳实践文档
