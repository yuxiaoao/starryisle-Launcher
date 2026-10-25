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

/**
 * StarryIsle maintains its own launcher changelog. Upstream Axolotl release
 * notes were removed on purpose: they describe another product and their titles
 * showed up as "Axolotl Launcher …" in the update history.
 */
export const launcherAnnouncements: readonly LauncherAnnouncement[] = [
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
                    'zh-CN':
                        '启动器更名为 StarryIsle，安装器、深链接与界面文案同步更新。',
                },
                {
                    'en-US': 'Reworked the default home layout and the widget picker.',
                    'zh-CN': '重新整理了首页默认布局与组件选择器。',
                },
                {
                    'en-US': 'Merged the latest upstream fixes for downloads, worlds and performance.',
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

export function getAnnouncementById(id: string) {
    return launcherAnnouncements.find((announcement) => announcement.id === id)
}

export function getLocalizedAnnouncementText(
    text: LocalizedAnnouncementText,
    locale: string,
): string {
    return locale === 'zh-CN' ? text['zh-CN'] : text['en-US']
}
