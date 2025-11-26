#!/bin/bash

# SSE聊天接口测试脚本

# 配置
API_BASE_URL="${API_BASE_URL:-http://localhost:8080/api}"
AGENT_ID="${AGENT_ID:-your-agent-id}"
TOKEN="${TOKEN:-your-token}"

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== SSE聊天接口测试 ===${NC}\n"

# 测试1: 流式聊天
echo -e "${GREEN}测试1: 流式聊天${NC}"
echo "发送消息: '你好，请介绍一下你自己'"
echo ""

curl -N -X POST "${API_BASE_URL}/agents/${AGENT_ID}/chat/stream" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "message": "你好，请介绍一下你自己",
    "stream": true
  }' 2>/dev/null | while IFS= read -r line; do
    if [[ $line == data:* ]]; then
      # 提取JSON数据
      json_data="${line#data: }"
      
      # 解析type字段
      chunk_type=$(echo "$json_data" | grep -o '"type":"[^"]*"' | cut -d'"' -f4)
      
      case "$chunk_type" in
        "content")
          # 提取并显示内容
          content=$(echo "$json_data" | grep -o '"content":"[^"]*"' | cut -d'"' -f4)
          echo -n "$content"
          ;;
        "done")
          echo ""
          echo -e "\n${GREEN}✓ 流式响应完成${NC}"
          # 显示元数据
          echo "$json_data" | grep -o '"metadata":{[^}]*}' | sed 's/"metadata":/元数据: /'
          ;;
        "error")
          echo ""
          error=$(echo "$json_data" | grep -o '"error":"[^"]*"' | cut -d'"' -f4)
          echo -e "${RED}✗ 错误: $error${NC}"
          ;;
      esac
    fi
  done

echo ""
echo -e "${BLUE}=== 测试完成 ===${NC}"

# 使用说明
cat << 'EOF'

使用方法:
---------

1. 设置环境变量:
   export API_BASE_URL="http://localhost:8080/api"
   export AGENT_ID="your-agent-uuid"
   export TOKEN="your-jwt-token"

2. 运行测试:
   bash test_sse_chat.sh

3. 或者直接指定参数:
   API_BASE_URL="http://localhost:8080/api" \
   AGENT_ID="agent-uuid" \
   TOKEN="jwt-token" \
   bash test_sse_chat.sh

测试非流式接口对比:
------------------
curl -X POST "${API_BASE_URL}/agents/${AGENT_ID}/chat" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "你好",
    "stream": false
  }'

EOF
