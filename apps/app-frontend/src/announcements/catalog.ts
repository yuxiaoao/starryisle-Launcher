export type AnnouncementLocale = 'en-US' | 'zh-CN'

export type AnnouncementChangeType =
    'added' | 'changed' | 'deprecated' | 'removed' | 'fixed' | 'security'

export type LocalizedAnnouncementText = Readonly<Record<AnnouncementLocale, string>>

export type AnnouncementChange = LocalizedAnnouncementText

export type LauncherAnnouncement = {
    readonly id: string
    readonly version: string
    readonly publishedAt: string
    readonly title: LocalizedAnnouncementText
    readonly changes: Readonly<
        Partial<Record<AnnouncementChangeType, readonly AnnouncementChange[]>>
    >
    readonly notes?: LocalizedAnnouncementText
    readonly externalUrl?: string
}

export const ANNOUNCEMENT_CHANGE_TYPES: readonly AnnouncementChangeType[] = [
    'added',
    'changed',
    'deprecated',
    'removed',
    'fixed',
    'security',
]

export const launcherAnnouncements: readonly LauncherAnnouncement[] = [
    {
        id: 'launcher-1.0.8',
        version: '1.0.8',
        publishedAt: '2026-11-07',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.8',
            'zh-CN': 'StarryIsle Launcher 1.0.8',
        },
        changes: {
            added: [
                {
                    'en-US': 'An advancements tab on every instance, reading the saved worlds.',
                    'zh-CN': '每个实例新增「成就」标签，读取存档里的进度。',
                },
                {
                    'en-US':
                        'A statistics page with playtime, library size and achievement progress.',
                    'zh-CN': '新增统计页面：游玩时长、整合包规模与成就进度。',
                },
            ],
            fixed: [
                {
                    'en-US':
                        'The hue slider shows its gradient again and matches the accent slider.',
                    'zh-CN': '色相滑块恢复彩虹渐变，与强调色滑块一致。',
                },
                {
                    'en-US': 'The interface hue now tints the nav rail and the title bar.',
                    'zh-CN': '界面色相现在真正作用于导航栏与顶栏。',
                },
            ],
        },
    },

    {
        id: 'launcher-1.0.7',
        version: '1.0.7',
        publishedAt: '2026-11-07',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.7',
            'zh-CN': 'StarryIsle Launcher 1.0.7',
        },
        changes: {
            added: [
                {
                    'en-US':
                        'An interface hue section sits between the colour theme and the accent colour.',
                    'zh-CN': '「色相」独立成区，位于色彩主题与强调色之间。',
                },
            ],
            changed: [
                {
                    'en-US':
                        'Improved accessibility with clearer labels for icon-only controls, notification announcements, and more consistent dialog focus.',
                    'zh-CN': '优化无障碍体验，为纯图标控件补充标签，改进通知播报和对话框焦点处理。',
                },
                {
                    'en-US':
                        'Improved narrow-window layouts with responsive pagination and scrollable content.',
                    'zh-CN': '优化窄窗口布局，改善分页栏和页面滚动表现。',
                },
                {
                    'en-US': 'Improved project translation by batching Bili Index requests.',
                    'zh-CN': '优化项目翻译，批量处理 Bili Index 翻译请求。',
                },
                {
                    'en-US':
                        'Improved Windows installation by detecting outdated WebView2 runtimes and providing update guidance.',
                    'zh-CN': '优化 Windows 安装流程，检测过旧的 WebView2 运行时并提供更新指引。',
                },
            ],
            fixed: [
                {
                    'en-US': 'The calendar shows up in the sidebar again.',
                    'zh-CN': '日历重新显示在右侧栏。',
                },
                {
                    'en-US': 'The hue control only appears on the appearance page.',
                    'zh-CN': '色相只在外观页面出现，不再出现在首页与导航、内容与下载页。',
                },
                {
                    'en-US': 'The hue slider matches the accent colour slider.',
                    'zh-CN': '色相滑块与强调色滑块样式一致。',
                },
                {
                    'en-US':
                        'Fixed the missing Data Packs category in Discover for Minecraft 26.x instances, including vanilla instances.',
                    'zh-CN':
                        '修复 Minecraft 26.x 实例（包括原版实例）在发现页缺少“数据包”分类的问题。',
                },
                {
                    'en-US':
                        'Fixed newer NeoForge instances failing to launch from external game directories with an installation corruption error.',
                    'zh-CN': '修复新版 NeoForge 实例使用外部游戏目录时，启动报安装损坏错误的问题。',
                },
                {
                    'en-US':
                        'Fixed Home incorrectly showing worlds as in use and disabling their play buttons when no game was running.',
                    'zh-CN': '修复未运行游戏时，主页误报世界正在使用中并禁用启动按钮的问题。',
                },
                {
                    'en-US':
                        'Fixed missing or unavailable instances blocking Home or leaving the launcher on an invalid instance page.',
                    'zh-CN': '修复实例缺失或不可用时阻塞主页，或使启动器停留在无效实例页面的问题。',
                },
                {
                    'en-US':
                        'Fixed launcher upgrades selecting the wrong database channel and preventing Home from loading across channels.',
                    'zh-CN':
                        '修复启动器升级时数据库通道选择错误，以及跨通道实例选择阻塞主页加载的问题。',
                },
                {
                    'en-US':
                        'Fixed cape previews failing to load because of WebView cross-origin restrictions.',
                    'zh-CN': '修复披风预览受 WebView 跨域限制而无法加载的问题。',
                },
                {
                    'en-US':
                        'Fixed automatic garbage collector selection when launching Minecraft from the system tray.',
                    'zh-CN': '修复从系统托盘启动 Minecraft 时自动垃圾回收器选择异常的问题。',
                },
                {
                    'en-US':
                        'Fixed Minecraft 26.3 authentication when using the Fallen authentication proxy.',
                    'zh-CN': '修复使用 Fallen 验证代理时 Minecraft 26.3 的验证问题。',
                },
                {
                    'en-US':
                        'Fixed malformed Modrinth project identities preventing other installed content from being checked for updates.',
                    'zh-CN': '修复无效 Modrinth 项目标识阻塞其他已安装内容更新检查的问题。',
                },
                {
                    'en-US':
                        'Fixed dependency refresh not using the current resource service and allowed cleanup of missing resource backups.',
                    'zh-CN': '修复依赖刷新未使用当前资源服务的问题，并允许清理已缺失的资源备份。',
                },
                {
                    'en-US':
                        'Fixed MRPack installation failing to stage overrides with long paths.',
                    'zh-CN': '修复 MRPack 安装时无法暂存长路径覆盖文件的问题。',
                },
                {
                    'en-US':
                        'Fixed browser form history suggestions appearing in the command generator\'s "Command to run" field after restarting the launcher.',
                    'zh-CN': '修复重启启动器后，指令生成器弹出浏览器表单历史建议的问题。',
                },
                {
                    'en-US': 'Fixed sidebar icons being draggable as links or images.',
                    'zh-CN': '修复侧边栏图标可被拖出为链接或图片的问题。',
                },
                {
                    'en-US':
                        'Fixed localized system font family names not being displayed in font selection.',
                    'zh-CN': '修复字体选择中未显示系统字体的本地化名称的问题。',
                },
                {
                    'en-US': 'Fixed compact project card actions covering the download count.',
                    'zh-CN': '修复紧凑项目卡片的操作按钮遮挡下载量的问题。',
                },
            ],
            security: [
                {
                    'en-US':
                        'Added permission checks and isolation for theme pack pages, storage, and network access.',
                    'zh-CN': '为主题包页面、存储和网络访问加入权限检查与隔离。',
                },
                {
                    'en-US':
                        'Fixed MRPack overrides with paths that differ only by letter case being applied to the same destination on case-insensitive systems.',
                    'zh-CN':
                        '修复大小写不敏感系统中，仅字母大小写不同的 MRPack 覆盖文件路径可能写入同一目标的问题。',
                },
            ],
        },
    },

    {
        id: 'launcher-1.0.6',
        version: '1.0.6',
        publishedAt: '2026-11-06',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.6',
            'zh-CN': 'StarryIsle Launcher 1.0.6',
        },
        changes: {
            changed: [
                {
                    'en-US':
                        'The launcher now checks this repository for updates instead of the upstream update server.',
                    'zh-CN': '启动器改为在本仓库检查更新，不再访问上游的更新服务器。',
                },
            ],
            fixed: [
                {
                    'en-US': 'Update checks no longer offer upstream builds.',
                    'zh-CN': '检查更新时不再提供上游版本的安装包。',
                },
            ],
        },
    },

    {
        id: 'launcher-1.0.5',
        version: '1.0.5',
        publishedAt: '2026-11-06',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.5',
            'zh-CN': 'StarryIsle Launcher 1.0.5',
        },
        changes: {
            added: [
                {
                    'en-US': 'Advancement pages for every instance, read from your saved worlds.',
                    'zh-CN': '每个实例的成就页面，直接读取存档里的进度。',
                },
                {
                    'en-US':
                        'Statistics page with total playtime, library size and achievement progress.',
                    'zh-CN': '统计页面：总游玩时长、整合包规模和成就进度。',
                },
                {
                    'en-US':
                        'Screenshot bundles: export a selection with its notes and import it elsewhere.',
                    'zh-CN': '截图包：连同备注一起导出选中截图，并能导入到别处。',
                },
                {
                    'en-US':
                        'Interface hue control that tints the navigation rail and the title bar.',
                    'zh-CN': '界面色相控制，可给导航栏和顶栏着色。',
                },
            ],
            changed: [
                {
                    'en-US': 'The home layout follows the original StarryIsle arrangement again.',
                    'zh-CN': '首页恢复为 StarryIsle 原本的布局。',
                },
                {
                    'en-US': 'The calendar sits between play insights and the daily challenge.',
                    'zh-CN': '日历回到了游玩洞察与每日小挑战之间。',
                },
            ],
            fixed: [
                {
                    'en-US':
                        'Product name, icon, deep link and update feed carry the StarryIsle identity.',
                    'zh-CN': '产品名称、图标、深链接和更新源均为 StarryIsle 身份。',
                },
                {
                    'en-US': 'Play insights show the all-time total above the weekly figure.',
                    'zh-CN': '游玩洞察在本周数据之上显示历史总时长。',
                },
            ],
        },
    },

    {
        id: 'launcher-1.0.4',
        version: '1.0.4',
        publishedAt: '2026-11-06',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.4',
            'zh-CN': 'StarryIsle Launcher 1.0.4',
        },
        changes: {
            changed: [
                {
                    'en-US':
                        'Rebuilt on upstream Axolotl 1.11.0-beta.2, so the launcher follows upstream directly again instead of a snapshot commit.',
                    'zh-CN':
                        '重建于上游 Axolotl 1.11.0-beta.2，启动器重新直接跟随上游，不再依赖快照提交。',
                },
                {
                    'en-US':
                        'Weather, the screenshot timeline and screenshot notes are available again on the new home dashboard.',
                    'zh-CN': '天气、截图时间线、截图备注已在新首页上恢复。',
                },
            ],
            fixed: [
                {
                    'en-US':
                        'Future upstream syncs apply only the new delta instead of moving the whole tree.',
                    'zh-CN': '以后同步上游只应用增量，不再整体搬运文件。',
                },
            ],
        },
    },
    {
        id: 'launcher-1.0.1',
        version: '1.0.1',
        publishedAt: '2026-10-25',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.1',
            'zh-CN': 'StarryIsle Launcher 1.0.1',
        },
        changes: {
            added: [
                {
                    'en-US':
                        'Automatic updates from this project\u2019s own GitHub releases, signed with a StarryIsle key.',
                    'zh-CN':
                        '新增自动更新：从本项目的 GitHub Releases 检查并安装新版本，使用 StarryIsle 专属密钥签名。',
                },
                {
                    'en-US': 'Total play time is shown above the weekly figure in Play insights.',
                    'zh-CN': '游玩洞察新增「总游玩」时长，位于「本周游玩」上方。',
                },
            ],
        },
    },
    {
        id: 'launcher-1.0.0',
        version: '1.0.0',
        publishedAt: '2026-10-24',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.0',
            'zh-CN': 'StarryIsle Launcher 1.0.0',
        },
        changes: {
            added: [
                {
                    'en-US':
                        'Weather widget with a built-in Chinese city list and manual city search.',
                    'zh-CN': '新增天气组件：内置中国城市列表，支持手动搜索城市。',
                },
                {
                    'en-US':
                        'Interface hue control for the nav rail and title bar, including hex input.',
                    'zh-CN': '新增界面色调：可为左侧导航栏与顶栏着色，支持十六进制取色。',
                },
            ],
            changed: [
                {
                    'en-US':
                        'Rebranded the launcher as StarryIsle, including the installer, deep link and every interface label.',
                    'zh-CN': '启动器更名为 StarryIsle，安装器、深链接与界面文案同步更新。',
                },
                {
                    'en-US': 'Reworked the default home layout and the widget picker.',
                    'zh-CN': '重新整理了首页默认布局与组件选择器。',
                },
                {
                    'en-US':
                        'Merged the latest upstream fixes for downloads, worlds and performance.',
                    'zh-CN': '合并上游最新修复：下载、世界列表与性能改进。',
                },
            ],
            removed: [
                {
                    'en-US': 'Automatic updates that connected to the Axolotl update service.',
                    'zh-CN': '移除了连接 Axolotl 更新服务的自动更新功能。',
                },
            ],
        },
        notes: {
            'en-US': 'New versions are published on this project\u2019s GitHub releases.',
            'zh-CN': '新版本只在本项目的 GitHub Releases 页面发布。',
        },
    },
]

export function getAnnouncementByVersion(version: string | null | undefined) {
    if (!version) return undefined
    return launcherAnnouncements.find((announcement) => announcement.version === version)
}

export function getAnnouncements(): readonly LauncherAnnouncement[] {
    return launcherAnnouncements
}

export function getLocalizedAnnouncementText(
    text: LocalizedAnnouncementText,
    locale: string,
): string {
    return locale === 'zh-CN' ? text['zh-CN'] : text['en-US']
}
