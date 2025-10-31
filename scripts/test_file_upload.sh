#!/bin/bash

# 文件上传功能测试脚本

set -e

BASE_URL="http://localhost:8080"
API_URL="${BASE_URL}/api"

echo "=== 文件上传功能测试 ==="
echo ""

# 1. 登录获取token
echo "1. 登录获取认证令牌..."
LOGIN_RESPONSE=$(curl -s -X POST "${API_URL}/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "test-tenant",
    "username": "admin",
    "password": "password"
  }')

TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
  echo "❌ 登录失败"
  echo "响应: $LOGIN_RESPONSE"
  exit 1
fi

echo "✅ 登录成功"
echo ""

# 2. 创建测试文件
echo "2. 创建测试文件..."
TEST_FILE="/tmp/test_upload.txt"
echo "This is a test file for upload functionality" > $TEST_FILE
echo "✅ 测试文件已创建: $TEST_FILE"
echo ""

# 3. 上传文件
echo "3. 上传文件..."
UPLOAD_RESPONSE=$(curl -s -X POST "${API_URL}/files/upload" \
  -H "Authorization: Bearer ${TOKEN}" \
  -F "file=@${TEST_FILE}")

echo "上传响应: $UPLOAD_RESPONSE"
echo ""

# 4. 解析文件URL
FILE_URL=$(echo $UPLOAD_RESPONSE | grep -o '"url":"[^"]*' | cut -d'"' -f4)

if [ -z "$FILE_URL" ]; then
  echo "❌ 文件上传失败"
  exit 1
fi

echo "✅ 文件上传成功"
echo "文件URL: $FILE_URL"
echo ""

# 5. 下载文件验证
echo "4. 验证文件下载..."
DOWNLOAD_RESPONSE=$(curl -s "$FILE_URL")

if [ "$DOWNLOAD_RESPONSE" == "This is a test file for upload functionality" ]; then
  echo "✅ 文件下载验证成功"
else
  echo "❌ 文件内容不匹配"
  echo "期望: This is a test file for upload functionality"
  echo "实际: $DOWNLOAD_RESPONSE"
  exit 1
fi

echo ""
echo "=== 所有测试通过 ✅ ==="

# 清理
rm -f $TEST_FILE
