# 国际化功能快速启动

## 🚀 快速开始

### 1. 安装依赖（已完成）

依赖已经安装，包括：
- i18next
- react-i18next
- i18next-browser-languagedetector
- js-cookie
- @types/js-cookie

### 2. 启动开发服务器

```bash
cd frontend
npm run dev
```

### 3. 测试语言切换

1. 打开浏览器访问 `http://localhost:5173`
2. 在登录页面右上角找到语言切换按钮（地球图标）
3. 点击切换中英文
4. 登录后，在主界面 Header 右侧也有语言切换按钮

## 📁 关键文件

```
frontend/src/
├── i18n/
│   ├── config.ts              # i18n 配置
│   └── locales/
│       ├── en.json            # 英文翻译
│       └── zh.json            # 中文翻译
├── components/common/
│   └── LanguageSwitcher.tsx   # 语言切换组件
└── main.tsx                   # 已导入 i18n 配置
```

## 🎯 已翻译的页面

- ✅ 登录页面
- ✅ 主界面 Header
- ✅ Agent 列表页面

## 💡 如何添加新翻译

### 步骤 1：在翻译文件中添加键值对

**en.json**:
```json
{
  "myFeature": {
    "title": "My Feature"
  }
}
```

**zh.json**:
```json
{
  "myFeature": {
    "title": "我的功能"
  }
}
```

### 步骤 2：在组件中使用

```tsx
import { useTranslation } from 'react-i18next';

function MyComponent() {
  const { t } = useTranslation();
  return <h1>{t('myFeature.title')}</h1>;
}
```

## 🔍 验证功能

### 检查 Cookie
1. 打开浏览器开发者工具（F12）
2. 进入 Application > Cookies
3. 查找 `app_language` cookie
4. 值应该是 `en` 或 `zh`

### 测试持久化
1. 选择一种语言
2. 刷新页面
3. 语言应该保持不变

### 测试浏览器语言检测
1. 清除 `app_language` cookie
2. 刷新页面
3. 应该显示浏览器默认语言（中文浏览器显示中文，其他显示英文）

## 📚 更多文档

- [完整使用指南](./I18N_GUIDE.md)
- [测试指南](./LANGUAGE_TESTING.md)
- [实现总结](./I18N_IMPLEMENTATION_SUMMARY.md)

## ⚠️ 注意事项

1. 语言切换是实时的，无需刷新页面
2. Cookie 有效期为 365 天
3. 支持的语言：`en`（英文）、`zh`（中文）
4. 默认语言：英文

## 🐛 常见问题

**Q: 语言切换后没有变化？**
A: 检查翻译文件中是否有对应的键值，确认组件使用了 `t()` 函数。

**Q: 刷新后语言恢复默认？**
A: 检查浏览器是否允许 Cookie，查看开发者工具中 Cookie 是否正确设置。

**Q: 某些文本没有翻译？**
A: 该文本可能还没有添加到翻译文件中，需要手动添加。

## 🎉 完成！

现在你可以开始使用国际化功能了！如有问题，请参考详细文档。
