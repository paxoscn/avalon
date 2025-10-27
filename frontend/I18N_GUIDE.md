# 国际化 (i18n) 使用指南

## 概述

前端应用现已支持中英双语切换功能。语言选择基于以下优先级：

1. **Cookie 存储的语言偏好** - 用户手动选择的语言会保存在 Cookie 中（有效期 365 天）
2. **浏览器默认语言** - 如果没有 Cookie，则检测浏览器语言设置
3. **默认语言** - 如果以上都不可用，默认使用英语

## 功能特性

- ✅ 支持中文（zh）和英文（en）
- ✅ 登录页面右上角有语言切换按钮
- ✅ 主界面 Header 中有语言切换按钮
- ✅ 语言选择自动保存到 Cookie
- ✅ 自动检测浏览器语言
- ✅ 实时切换，无需刷新页面

## 技术实现

### 依赖包

```json
{
  "i18next": "^23.x",
  "react-i18next": "^14.x",
  "i18next-browser-languagedetector": "^7.x",
  "js-cookie": "^3.x",
  "@types/js-cookie": "^3.x"
}
```

### 文件结构

```
frontend/src/
├── i18n/
│   ├── config.ts              # i18n 配置和初始化
│   └── locales/
│       ├── en.json            # 英文翻译
│       └── zh.json            # 中文翻译
└── components/
    └── common/
        └── LanguageSwitcher.tsx  # 语言切换组件
```

## 使用方法

### 在组件中使用翻译

```tsx
import { useTranslation } from 'react-i18next';

function MyComponent() {
  const { t } = useTranslation();
  
  return (
    <div>
      <h1>{t('agents.title')}</h1>
      <p>{t('agents.description')}</p>
    </div>
  );
}
```

### 添加新的翻译

1. 在 `frontend/src/i18n/locales/en.json` 中添加英文翻译：
```json
{
  "myFeature": {
    "title": "My Feature",
    "description": "This is my feature"
  }
}
```

2. 在 `frontend/src/i18n/locales/zh.json` 中添加中文翻译：
```json
{
  "myFeature": {
    "title": "我的功能",
    "description": "这是我的功能"
  }
}
```

3. 在组件中使用：
```tsx
<h1>{t('myFeature.title')}</h1>
<p>{t('myFeature.description')}</p>
```

### 添加语言切换按钮

语言切换组件已经在以下位置添加：
- 登录页面（右上角）
- 主界面 Header（用户菜单左侧）

如需在其他位置添加：
```tsx
import { LanguageSwitcher } from '../components/common';

function MyPage() {
  return (
    <div>
      <LanguageSwitcher />
    </div>
  );
}
```

## Cookie 配置

语言偏好存储在名为 `app_language` 的 Cookie 中：
- 有效期：365 天
- 值：`en` 或 `zh`

## 浏览器语言检测

系统会自动检测浏览器语言：
- 如果浏览器语言以 `zh` 开头（如 `zh-CN`, `zh-TW`），则使用中文
- 其他情况使用英文

## 已翻译的页面

目前已完成翻译的页面和组件：
- ✅ 登录页面
- ✅ 主界面 Header
- ✅ 侧边栏菜单（Sidebar）
- ✅ Agent 列表页面
- ✅ 通用组件（按钮、分页等）

## 待完成

其他页面的翻译可以按照相同的模式逐步添加：
1. 在翻译文件中添加对应的键值对
2. 在组件中使用 `useTranslation` hook
3. 用 `t()` 函数替换硬编码的文本

## 测试

1. 打开应用，默认应该显示浏览器语言或英文
2. 点击语言切换按钮，选择另一种语言
3. 刷新页面，语言应该保持不变（从 Cookie 读取）
4. 清除浏览器 Cookie，刷新页面，应该恢复到浏览器默认语言
