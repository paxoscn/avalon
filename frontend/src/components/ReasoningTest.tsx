import React, { useState, useEffect } from 'react';

/**
 * 测试组件 - 用于验证思考过程 UI 是否正常工作
 * 
 * 使用方法：
 * import { ReasoningTest } from './components/ReasoningTest';
 * <ReasoningTest />
 */
export function ReasoningTest() {
  const [currentReasoning, setCurrentReasoning] = useState('');
  const [currentResponse, setCurrentResponse] = useState('');
  const [isStreaming, setIsStreaming] = useState(false);

  const simulateStreaming = () => {
    setIsStreaming(true);
    setCurrentReasoning('');
    setCurrentResponse('');

    // 模拟思考过程
    const reasoningSteps = [
      '正在分析您的问题...',
      '正在分析您的问题...\n检索相关知识库...',
      '正在分析您的问题...\n检索相关知识库...\n整理回答思路...',
    ];

    let reasoningIndex = 0;
    const reasoningInterval = setInterval(() => {
      if (reasoningIndex < reasoningSteps.length) {
        setCurrentReasoning(reasoningSteps[reasoningIndex]);
        reasoningIndex++;
      } else {
        clearInterval(reasoningInterval);
        
        // 思考过程结束，清空
        setTimeout(() => {
          setCurrentReasoning('');
          
          // 开始显示回复
          const responseText = '根据您的问题，我的理解是：这是一个很好的问题。让我详细为您解答。';
          let responseIndex = 0;
          
          const responseInterval = setInterval(() => {
            if (responseIndex < responseText.length) {
              setCurrentResponse(responseText.slice(0, responseIndex + 1));
              responseIndex++;
            } else {
              clearInterval(responseInterval);
              setIsStreaming(false);
            }
          }, 50);
        }, 500);
      }
    }, 1000);
  };

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="bg-white rounded-lg shadow-lg p-6">
        <h1 className="text-2xl font-bold mb-4">思考过程 UI 测试</h1>
        
        <button
          onClick={simulateStreaming}
          disabled={isStreaming}
          className="px-6 py-3 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors font-medium disabled:bg-gray-300 disabled:cursor-not-allowed mb-6"
        >
          {isStreaming ? '流式响应中...' : '开始测试'}
        </button>

        <div className="space-y-4">
          <div className="border-t pt-4">
            <h2 className="text-lg font-semibold mb-2">状态信息：</h2>
            <div className="bg-gray-50 p-4 rounded-lg space-y-2 text-sm font-mono">
              <div>isStreaming: <span className="font-bold">{isStreaming ? 'true' : 'false'}</span></div>
              <div>currentReasoning: <span className="font-bold">{currentReasoning ? `"${currentReasoning.slice(0, 30)}..."` : 'null'}</span></div>
              <div>currentResponse: <span className="font-bold">{currentResponse ? `"${currentResponse.slice(0, 30)}..."` : 'null'}</span></div>
            </div>
          </div>

          <div className="border-t pt-4">
            <h2 className="text-lg font-semibold mb-2">UI 预览：</h2>
            
            {isStreaming && (
              <div className="flex items-start space-x-3 animate-fade-in">
                <div className="flex-shrink-0 w-8 h-8 bg-gradient-to-br from-blue-400 to-blue-600 rounded-full flex items-center justify-center text-white font-semibold shadow-md">
                  🤖
                </div>
                <div className="flex-1 space-y-3">
                  {/* 思考过程 */}
                  {currentReasoning && (
                    <div className="bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-lg p-4 shadow-sm">
                      <div className="flex items-center space-x-2 mb-2">
                        <svg className="w-5 h-5 text-amber-600 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        <span className="text-sm font-semibold text-amber-700">💭 思考过程</span>
                      </div>
                      <div className="text-sm text-amber-900 whitespace-pre-wrap leading-relaxed">
                        {currentReasoning}
                        <span className="inline-block w-1 h-4 bg-amber-600 animate-pulse ml-1 align-middle"></span>
                      </div>
                    </div>
                  )}
                  
                  {/* 回复内容 - 只在思考过程结束后显示 */}
                  {!currentReasoning && currentResponse && (
                    <div className="bg-white rounded-lg shadow-sm p-4 border border-gray-100">
                      <div className="text-gray-800 whitespace-pre-wrap leading-relaxed">
                        {currentResponse}
                        <span className="inline-block w-1 h-4 bg-blue-500 animate-pulse ml-1 align-middle"></span>
                      </div>
                    </div>
                  )}
                  
                  {/* 加载状态 */}
                  {!currentReasoning && !currentResponse && (
                    <div className="bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-200 rounded-lg p-4 shadow-sm">
                      <div className="flex items-center space-x-2">
                        <svg className="w-5 h-5 text-amber-600 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        <span className="text-sm font-semibold text-amber-700">💭 正在思考...</span>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {!isStreaming && (
              <div className="text-gray-500 text-center py-8">
                点击"开始测试"按钮查看效果
              </div>
            )}
          </div>

          <div className="border-t pt-4">
            <h2 className="text-lg font-semibold mb-2">预期行为：</h2>
            <ol className="list-decimal list-inside space-y-2 text-sm text-gray-700">
              <li>点击"开始测试"按钮</li>
              <li>显示"💭 正在思考..."加载状态</li>
              <li>显示思考过程（逐步更新）</li>
              <li>思考过程结束后消失</li>
              <li>显示回复内容（逐字显示）</li>
              <li>回复完成</li>
            </ol>
          </div>

          <div className="border-t pt-4">
            <h2 className="text-lg font-semibold mb-2">如果看不到思考过程：</h2>
            <ul className="list-disc list-inside space-y-2 text-sm text-gray-700">
              <li>检查浏览器控制台是否有错误</li>
              <li>确认 Tailwind CSS 已正确加载</li>
              <li>检查 CSS 动画是否被禁用</li>
              <li>尝试刷新页面</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ReasoningTest;
