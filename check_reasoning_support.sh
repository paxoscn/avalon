#!/bin/bash

echo "🔍 检查 reasoning_content 支持情况"
echo "======================================"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 1. 检查前端代码
echo "📁 检查前端代码..."
if grep -q "reasoning_content" frontend/src/hooks/useAgentChatStream.ts; then
    echo -e "${GREEN}✅ useAgentChatStream.ts 支持 reasoning_content${NC}"
else
    echo -e "${RED}❌ useAgentChatStream.ts 不支持 reasoning_content${NC}"
fi

if grep -q "currentReasoning" frontend/src/components/AgentChatStream.tsx; then
    echo -e "${GREEN}✅ AgentChatStream.tsx 支持 currentReasoning${NC}"
else
    echo -e "${RED}❌ AgentChatStream.tsx 不支持 currentReasoning${NC}"
fi

if grep -q "onReasoning" frontend/src/services/chat.service.ts; then
    echo -e "${GREEN}✅ chat.service.ts 支持 onReasoning${NC}"
else
    echo -e "${RED}❌ chat.service.ts 不支持 onReasoning${NC}"
fi

echo ""

# 2. 检查后端代码
echo "📁 检查后端代码..."
if grep -q "reasoning_content" src/domain/services/llm_service.rs; then
    echo -e "${GREEN}✅ llm_service.rs 定义了 reasoning_content${NC}"
else
    echo -e "${RED}❌ llm_service.rs 没有定义 reasoning_content${NC}"
fi

if grep -q "reasoning_content" src/infrastructure/llm/streaming.rs; then
    echo -e "${GREEN}✅ streaming.rs 解析 reasoning_content${NC}"
else
    echo -e "${RED}❌ streaming.rs 不解析 reasoning_content${NC}"
fi

if grep -q "reasoning_content" src/application/dto/agent_dto.rs; then
    echo -e "${GREEN}✅ agent_dto.rs 包含 reasoning_content${NC}"
else
    echo -e "${RED}❌ agent_dto.rs 不包含 reasoning_content${NC}"
fi

echo ""

# 3. 检查演示页面
echo "📁 检查演示页面..."
if [ -f "frontend/src/examples/chat-demo.html" ]; then
    echo -e "${GREEN}✅ 演示页面存在${NC}"
    echo "   运行: open frontend/src/examples/chat-demo.html"
else
    echo -e "${RED}❌ 演示页面不存在${NC}"
fi

echo ""

# 4. 支持的模型列表
echo "📋 支持 reasoning_content 的模型："
echo -e "${GREEN}✅ o1-preview${NC}"
echo -e "${GREEN}✅ o1-mini${NC}"
echo -e "${GREEN}✅ o1 (如果可用)${NC}"
echo ""
echo "❌ 不支持的模型："
echo "   - gpt-4"
echo "   - gpt-4-turbo"
echo "   - gpt-3.5-turbo"
echo "   - Claude 系列"
echo ""

# 5. 调试建议
echo "🔧 调试建议："
echo ""
echo "1. 检查模型配置："
echo "   SELECT name, llm_config->>'model' as model FROM agents;"
echo ""
echo "2. 查看浏览器控制台（F12）："
echo "   - 查找 'Received chunk:' 日志"
echo "   - 查找 'Reasoning update:' 日志"
echo ""
echo "3. 检查网络请求："
echo "   - 打开 Network 标签"
echo "   - 找到 /chat/stream 请求"
echo "   - 查看 Response 中是否有 reasoning_content"
echo ""
echo "4. 测试演示页面："
echo "   open frontend/src/examples/chat-demo.html"
echo ""
echo "5. 阅读调试指南："
echo "   cat DEBUG_REASONING_CONTENT.md"
echo ""

# 6. 快速测试
echo "🧪 快速测试："
echo ""
echo "# 启动前端"
echo "cd frontend && npm run dev"
echo ""
echo "# 访问聊天页面"
echo "http://localhost:5173/agents/{agentId}/chat"
echo ""
echo "# 发送消息并检查控制台"
echo "# 应该看到："
echo "# - Received chunk: {...}"
echo "# - Reasoning update: ... (如果模型支持)"
echo "# - Content update: ..."
echo ""

echo "======================================"
echo "✅ 检查完成"
echo ""
echo "💡 提示："
echo "如果没有看到 reasoning_content，最可能的原因是："
echo "1. 使用的模型不支持（需要 o1-preview 或 o1-mini）"
echo "2. 模型配置不正确"
echo ""
echo "详细调试步骤请查看: DEBUG_REASONING_CONTENT.md"
