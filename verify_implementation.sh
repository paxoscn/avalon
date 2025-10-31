#!/bin/bash

echo "=========================================="
echo "Structured Output 功能实现验证"
echo "=========================================="
echo ""

echo "1. 检查编译状态..."
if cargo check 2>&1 | grep -q "error\[E"; then
    echo "❌ 编译失败"
    exit 1
else
    echo "✅ 编译成功"
fi
echo ""

echo "2. 检查关键文件..."
files=(
    "src/domain/services/llm_service.rs"
    "src/domain/services/node_executors.rs"
    "src/infrastructure/llm/providers/openai.rs"
    "src/application/services/integrated_llm_service.rs"
    "src/application/services/llm_integration_service.rs"
    "src/application/services/llm_application_service.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file"
    else
        echo "❌ $file 不存在"
    fi
done
echo ""

echo "3. 检查关键代码..."
echo ""

echo "3.1 检查 ResponseFormat 结构体..."
if grep -q "pub struct ResponseFormat" src/domain/services/llm_service.rs; then
    echo "✅ ResponseFormat 结构体已定义"
else
    echo "❌ ResponseFormat 结构体未找到"
fi

echo "3.2 检查 JsonSchema 结构体..."
if grep -q "pub struct JsonSchema" src/domain/services/llm_service.rs; then
    echo "✅ JsonSchema 结构体已定义"
else
    echo "❌ JsonSchema 结构体未找到"
fi

echo "3.3 检查 ChatRequest.response_format 字段..."
if grep -q "pub response_format: Option<ResponseFormat>" src/domain/services/llm_service.rs; then
    echo "✅ ChatRequest.response_format 字段已添加"
else
    echo "❌ ChatRequest.response_format 字段未找到"
fi

echo "3.4 检查 extract_structured_output 方法..."
if grep -q "fn extract_structured_output" src/domain/services/node_executors.rs; then
    echo "✅ extract_structured_output 方法已实现"
else
    echo "❌ extract_structured_output 方法未找到"
fi

echo "3.5 检查 OpenAIChatRequest.response_format 字段..."
if grep -q "response_format: Option<serde_json::Value>" src/infrastructure/llm/providers/openai.rs; then
    echo "✅ OpenAIChatRequest.response_format 字段已添加"
else
    echo "❌ OpenAIChatRequest.response_format 字段未找到"
fi

echo ""
echo "=========================================="
echo "验证完成！"
echo "=========================================="
echo ""

echo "功能说明："
echo "- 节点配置中的 structured_output 会被自动转换为 response_format"
echo "- response_format 会被传递给 OpenAI API"
echo "- 格式符合 OpenAI 的 json_schema 规范"
echo ""

echo "使用示例："
echo "在节点的 data 字段中添加："
echo '```json'
echo '"structured_output": {'
echo '  "schema": {'
echo '    "type": "object",'
echo '    "properties": {'
echo '      "passed": {'
echo '        "type": "boolean"'
echo '      }'
echo '    },'
echo '    "required": ["passed"],'
echo '    "additionalProperties": false'
echo '  }'
echo '}'
echo '```'
