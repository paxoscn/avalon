#!/bin/bash

# Agent发布功能测试脚本

BASE_URL="http://localhost:8080/api"
TOKEN="your_auth_token_here"

echo "=== Agent发布功能测试 ==="
echo ""

# 1. 创建一个新的agent
echo "1. 创建新agent..."
CREATE_RESPONSE=$(curl -s -X POST "$BASE_URL/agents" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "测试Agent",
    "system_prompt": "你是一个测试助手",
    "avatar": null,
    "greeting": "你好！",
    "additional_settings": null,
    "preset_questions": ["问题1", "问题2"],
    "knowledge_base_ids": [],
    "mcp_tool_ids": [],
    "flow_ids": []
  }')

AGENT_ID=$(echo $CREATE_RESPONSE | jq -r '.id')
echo "创建的Agent ID: $AGENT_ID"
echo "发布状态: $(echo $CREATE_RESPONSE | jq -r '.is_published')"
echo ""

# 2. 获取agent详情（应该显示未发布）
echo "2. 获取agent详情..."
curl -s -X GET "$BASE_URL/agents/$AGENT_ID" \
  -H "Authorization: Bearer $TOKEN" | jq '{id, name, is_published, published_at}'
echo ""

# 3. 发布agent
echo "3. 发布agent..."
curl -s -X POST "$BASE_URL/agents/$AGENT_ID/publish" \
  -H "Authorization: Bearer $TOKEN" \
  -w "\nHTTP Status: %{http_code}\n"
echo ""

# 4. 再次获取agent详情（应该显示已发布）
echo "4. 获取agent详情（发布后）..."
curl -s -X GET "$BASE_URL/agents/$AGENT_ID" \
  -H "Authorization: Bearer $TOKEN" | jq '{id, name, is_published, published_at}'
echo ""

# 5. 查询公共agent列表（应该包含刚发布的agent）
echo "5. 查询公共agent列表..."
curl -s -X GET "$BASE_URL/agents?page=1&limit=10" \
  -H "Authorization: Bearer $TOKEN" | jq '.items[] | {id, name, is_published}'
echo ""

# 6. 取消发布agent
echo "6. 取消发布agent..."
curl -s -X POST "$BASE_URL/agents/$AGENT_ID/unpublish" \
  -H "Authorization: Bearer $TOKEN" \
  -w "\nHTTP Status: %{http_code}\n"
echo ""

# 7. 再次获取agent详情（应该显示未发布）
echo "7. 获取agent详情（取消发布后）..."
curl -s -X GET "$BASE_URL/agents/$AGENT_ID" \
  -H "Authorization: Bearer $TOKEN" | jq '{id, name, is_published, published_at}'
echo ""

# 8. 查询我创建的agents（应该包含未发布的agent）
echo "8. 查询我创建的agents..."
curl -s -X GET "$BASE_URL/agents/created?page=1&limit=10" \
  -H "Authorization: Bearer $TOKEN" | jq '.items[] | {id, name, is_published}'
echo ""

# 9. 清理：删除测试agent
echo "9. 删除测试agent..."
curl -s -X DELETE "$BASE_URL/agents/$AGENT_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -w "\nHTTP Status: %{http_code}\n"
echo ""

echo "=== 测试完成 ==="
