# CC Switch 重构实施清单

> 用于跟踪重构进度的详细检查清单

**开始日期**: ___________
**预计完成**: ___________
**当前阶段**: ___________

---

## 📋 阶段 0: 准备阶段 (预计 1 天)

### 环境准备

- [ ] 创建新分支 `refactor/modernization`
- [ ] 创建备份标签 `git tag backup-before-refactor`
- [ ] 备份用户配置文件 `~/.cc-switch/config.json`
- [ ] 通知团队成员重构开始

### 依赖安装

```bash
pnpm add @tanstack/react-query
pnpm add react-hook-form @hookform/resolvers
pnpm add zod
pnpm add sonner
pnpm add next-themes
pnpm add @radix-ui/react-dialog @radix-ui/react-dropdown-menu
pnpm add @radix-ui/react-label @radix-ui/react-select
pnpm add @radix-ui/react-slot @radix-ui/react-switch @radix-ui/react-tabs
pnpm add class-variance-authority clsx tailwind-merge tailwindcss-animate
```

- [ ] 安装核心依赖 (上述命令)
- [ ] 验证依赖安装成功 `pnpm install`
- [ ] 验证编译通过 `pnpm typecheck`

### 配置文件

- [ ] 创建 `components.json`
- [ ] 更新 `tsconfig.json` 添加路径别名
- [ ] 更新 `vite.config.mts` 添加路径解析
- [ ] 验证开发服务器启动 `pnpm dev`

**完成时间**: ___________
**遇到的问题**: ___________

---

## 📋 阶段 1: 基础设施 (预计 2-3 天)

### 1.1 工具函数和基础组件

- [ ] 创建 `src/lib/utils.ts` (cn 函数)
- [ ] 创建 `src/components/ui/button.tsx`
- [ ] 创建 `src/components/ui/dialog.tsx`
- [ ] 创建 `src/components/ui/input.tsx`
- [ ] 创建 `src/components/ui/label.tsx`
- [ ] 创建 `src/components/ui/textarea.tsx`
- [ ] 创建 `src/components/ui/select.tsx`
- [ ] 创建 `src/components/ui/switch.tsx`
- [ ] 创建 `src/components/ui/tabs.tsx`
- [ ] 创建 `src/components/ui/sonner.tsx`
- [ ] 创建 `src/components/ui/form.tsx`

**测试**:
- [ ] 验证所有 UI 组件可以正常导入
- [ ] 创建一个测试页面验证组件样式

### 1.2 Query Client 设置

- [ ] 创建 `src/lib/query/queryClient.ts`
- [ ] 配置默认选项 (retry, staleTime 等)
- [ ] 导出 queryClient 实例

### 1.3 API 层

- [ ] 创建 `src/lib/api/providers.ts`
  - [ ] getAll
  - [ ] getCurrent
  - [ ] add
  - [ ] update
  - [ ] delete
  - [ ] switch
  - [ ] importDefault
  - [ ] updateTrayMenu

- [ ] 创建 `src/lib/api/settings.ts`
  - [ ] get
  - [ ] save

- [ ] 创建 `src/lib/api/mcp.ts`
  - [ ] getConfig
  - [ ] upsertServer
  - [ ] deleteServer

- [ ] 创建 `src/lib/api/index.ts` (聚合导出)

**测试**:
- [ ] 验证 API 调用不会出现运行时错误
- [ ] 确认类型定义正确

### 1.4 Query Hooks

- [ ] 创建 `src/lib/query/queries.ts`
  - [ ] useProvidersQuery
  - [ ] useSettingsQuery
  - [ ] useMcpConfigQuery

- [ ] 创建 `src/lib/query/mutations.ts`
  - [ ] useAddProviderMutation
  - [ ] useSwitchProviderMutation
  - [ ] useDeleteProviderMutation
  - [ ] useUpdateProviderMutation
  - [ ] useSaveSettingsMutation

- [ ] 创建 `src/lib/query/index.ts` (聚合导出)

**测试**:
- [ ] 在临时组件中测试每个 hook
- [ ] 验证 loading/error 状态正确
- [ ] 验证缓存和自动刷新工作

**完成时间**: ___________
**遇到的问题**: ___________

---

## 📋 阶段 2: 核心功能重构 (预计 3-4 天)

### 2.1 主题系统

- [ ] 创建 `src/components/theme-provider.tsx`
- [ ] 创建 `src/components/mode-toggle.tsx`
- [ ] 更新 `src/index.css` 添加主题变量
- [ ] 删除 `src/hooks/useDarkMode.ts`
- [ ] 更新所有组件使用新的主题系统

**测试**:
- [ ] 验证主题切换正常工作
- [ ] 验证系统主题跟随功能
- [ ] 验证主题持久化

### 2.2 更新 main.tsx

- [ ] 引入 QueryClientProvider
- [ ] 引入 ThemeProvider
- [ ] 添加 Toaster 组件
- [ ] 移除旧的 API 导入

**测试**:
- [ ] 验证应用可以正常启动
- [ ] 验证 Context 正确传递

### 2.3 重构 App.tsx

- [ ] 使用 useProvidersQuery 替代手动状态管理
- [ ] 移除所有 loadProviders 相关代码
- [ ] 移除手动 notification 状态
- [ ] 简化事件监听逻辑
- [ ] 更新对话框为新的 Dialog 组件

**目标**: 将 412 行代码减少到 ~100 行

**测试**:
- [ ] 验证供应商列表正常加载
- [ ] 验证切换 Claude/Codex 正常工作
- [ ] 验证事件监听正常工作

### 2.4 重构 ProviderList

- [ ] 创建 `src/components/providers/ProviderList.tsx`
- [ ] 使用 mutation hooks 处理操作
- [ ] 移除 onNotify prop
- [ ] 移除手动状态管理

**测试**:
- [ ] 验证供应商列表渲染
- [ ] 验证切换操作
- [ ] 验证删除操作

### 2.5 重构表单系统

- [ ] 创建 `src/lib/schemas/provider.ts` (Zod schema)
- [ ] 创建 `src/components/providers/ProviderForm.tsx`
  - [ ] 使用 react-hook-form
  - [ ] 使用 zodResolver
  - [ ] 字段级验证

- [ ] 创建 `src/components/providers/AddProviderDialog.tsx`
  - [ ] 使用新的 Dialog 组件
  - [ ] 集成 ProviderForm
  - [ ] 使用 useAddProviderMutation

- [ ] 创建 `src/components/providers/EditProviderDialog.tsx`
  - [ ] 使用新的 Dialog 组件
  - [ ] 集成 ProviderForm
  - [ ] 使用 useUpdateProviderMutation

**测试**:
- [ ] 验证表单验证正常工作
- [ ] 验证错误提示显示正确
- [ ] 验证提交操作成功
- [ ] 验证表单重置功能

### 2.6 清理旧组件

- [ ] 删除 `src/components/AddProviderModal.tsx`
- [ ] 删除 `src/components/EditProviderModal.tsx`
- [ ] 更新所有引用这些组件的地方

**完成时间**: ___________
**遇到的问题**: ___________

---

## 📋 阶段 3: 设置和辅助功能 (预计 2-3 天)

### 3.1 重构 SettingsDialog

- [ ] 创建 `src/components/settings/SettingsDialog.tsx`
  - [ ] 使用 Tabs 组件
  - [ ] 集成各个设置子组件

- [ ] 创建 `src/components/settings/GeneralSettings.tsx`
  - [ ] 语言设置
  - [ ] 配置目录设置
  - [ ] 其他通用设置

- [ ] 创建 `src/components/settings/AboutSection.tsx`
  - [ ] 版本信息
  - [ ] 更新检查
  - [ ] 链接

- [ ] 创建 `src/components/settings/ImportExportSection.tsx`
  - [ ] 导入功能
  - [ ] 导出功能

**目标**: 将 643 行拆分为 4-5 个小组件，每个 100-150 行

**测试**:
- [ ] 验证设置保存功能
- [ ] 验证导入导出功能
- [ ] 验证更新检查功能

### 3.2 重构通知系统

- [ ] 在所有 mutations 中使用 `toast` 替代 `showNotification`
- [ ] 移除 App.tsx 中的 notification 状态
- [ ] 移除自定义通知组件

**测试**:
- [ ] 验证成功通知显示
- [ ] 验证错误通知显示
- [ ] 验证通知自动消失

### 3.3 重构确认对话框

- [ ] 更新 `src/components/ConfirmDialog.tsx` 使用新的 Dialog
- [ ] 或者直接使用 shadcn/ui 的 AlertDialog

**测试**:
- [ ] 验证删除确认对话框
- [ ] 验证其他确认场景

**完成时间**: ___________
**遇到的问题**: ___________

---

## 📋 阶段 4: 清理和优化 (预计 1-2 天)

### 4.1 移除旧代码

- [x] 删除 `src/lib/styles.ts`
- [x] 从 `src/lib/tauri-api.ts` 移除 `window.api` 绑定
- [x] 精简 `src/lib/tauri-api.ts`，只保留事件监听相关
- [x] 删除或更新 `src/vite-env.d.ts` 中的过时类型

### 4.2 代码审查

- [ ] 检查所有 TODO 注释
- [x] 检查是否还有 `window.api` 调用
- [ ] 检查是否还有手动状态管理
- [x] 统一代码风格

### 4.3 类型检查

- [x] 运行 `pnpm typecheck` 确保无错误
- [x] 修复所有类型错误
- [x] 更新类型定义

### 4.4 性能优化

- [ ] 检查是否有不必要的重渲染
- [ ] 添加必要的 React.memo
- [ ] 优化 Query 缓存配置

**完成时间**: ___________
**遇到的问题**: ___________

---

## 📋 阶段 5: 测试和修复 (预计 2-3 天)

### 5.1 功能测试

#### 供应商管理
- [ ] 添加供应商 (Claude)
- [ ] 添加供应商 (Codex)
- [ ] 编辑供应商
- [ ] 删除供应商
- [ ] 切换供应商
- [ ] 导入默认配置

#### 应用切换
- [ ] Claude <-> Codex 切换
- [ ] 切换后数据正确加载
- [ ] 切换后托盘菜单更新

#### 设置
- [ ] 保存通用设置
- [ ] 切换语言
- [ ] 配置目录选择
- [ ] 导入配置
- [ ] 导出配置

#### UI 交互
- [ ] 主题切换 (亮色/暗色)
- [ ] 对话框打开/关闭
- [ ] 表单验证
- [ ] Toast 通知

#### MCP 管理
- [ ] 列表显示
- [ ] 添加 MCP
- [ ] 编辑 MCP
- [ ] 删除 MCP
- [ ] 启用/禁用 MCP

### 5.2 边界情况测试

- [ ] 空供应商列表
- [ ] 无效配置文件
- [ ] 网络错误
- [ ] 后端错误响应
- [ ] 并发操作
- [ ] 表单输入边界值

### 5.3 兼容性测试

- [ ] Windows 测试
- [ ] macOS 测试
- [ ] Linux 测试

### 5.4 性能测试

- [ ] 100+ 供应商加载速度
- [ ] 快速切换供应商
- [ ] 内存使用情况
- [ ] CPU 使用情况

### 5.5 Bug 修复

**Bug 列表** (发现后记录):

1. ___________
   - [ ] 已修复
   - [ ] 已验证

2. ___________
   - [ ] 已修复
   - [ ] 已验证

**完成时间**: ___________
**遇到的问题**: ___________

---

## 📋 最终检查

### 代码质量

- [ ] 所有 TypeScript 错误已修复
- [ ] 运行 `pnpm format` 格式化代码
- [ ] 运行 `pnpm typecheck` 通过
- [ ] 代码审查完成

### 文档更新

- [ ] 更新 `CLAUDE.md` 反映新架构
- [ ] 更新 `README.md` (如有必要)
- [ ] 添加 Migration Guide (可选)

### 性能基准

记录性能数据:

**旧版本**:
- 启动时间: _____ms
- 供应商加载: _____ms
- 内存占用: _____MB

**新版本**:
- 启动时间: _____ms
- 供应商加载: _____ms
- 内存占用: _____MB

### 代码统计

**代码行数对比**:

| 文件 | 旧版本 | 新版本 | 减少 |
|------|--------|--------|------|
| App.tsx | 412 | ~100 | -76% |
| tauri-api.ts | 712 | ~50 | -93% |
| ProviderForm.tsx | 271 | ~150 | -45% |
| settings 模块 | 1046 | ~470 (拆分) | -55% |
| **总计** | 2038 | ~700 | **-66%** |

---

## 📦 发布准备

### Pre-release 测试

- [ ] 创建 beta 版本 `v4.0.0-beta.1`
- [ ] 在测试环境验证
- [ ] 收集用户反馈

### 正式发布

- [ ] 合并到 main 分支
- [ ] 创建 Release Tag `v4.0.0`
- [ ] 更新 Changelog
- [ ] 发布 GitHub Release
- [ ] 通知用户更新

---

## 🚨 回滚触发条件

如果出现以下情况，考虑回滚:

- [ ] 重大功能无法使用
- [ ] 用户数据丢失
- [ ] 严重性能问题
- [ ] 无法修复的兼容性问题

**回滚命令**:
```bash
git reset --hard backup-before-refactor
# 或
git revert <commit-range>
```

---

## 📝 总结报告

### 成功指标

- [ ] 所有现有功能正常工作
- [ ] 代码量减少 40%+
- [ ] 无用户数据丢失
- [ ] 性能未下降

### 经验教训

**遇到的主要挑战**:
1. ___________
2. ___________
3. ___________

**解决方案**:
1. ___________
2. ___________
3. ___________

**未来改进**:
1. ___________
2. ___________
3. ___________

---

**重构完成日期**: ___________
**总耗时**: _____ 天
**参与人员**: ___________
