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
        id: 'launcher-1.0.2',
        version: '1.0.2',
        publishedAt: '2026-10-24',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.2',
            'zh-CN': 'StarryIsle Launcher 1.0.2',
        },
        changes: {
            changed: [
                {
                    'en-US':
                        'Every remaining Axolotl label in the interface now reads StarryIsle: the create instance dialog, settings, error messages and lab tools.',
                    'zh-CN':
                        '界面中残留的 Axolotl 文案统一改为 StarryIsle：创建实例对话框、设置页、错误提示与实验工具。',
                },
            ],
        },
    },
    {
        id: 'launcher-1.0.1',
        version: '1.0.1',
        publishedAt: '2026-10-24',
        title: {
            'en-US': 'StarryIsle Launcher 1.0.1',
            'zh-CN': 'StarryIsle Launcher 1.0.1',
        },
        changes: {
            removed: [
                {
                    'en-US': 'Automatic updates that connected to the Axolotl update service.',
                    'zh-CN': '移除了连接 Axolotl 更新服务的自动更新功能。',
                },
            ],
            fixed: [
                {
                    'en-US': 'The update log no longer lists Axolotl Launcher releases.',
                    'zh-CN': '更新日志不再显示 Axolotl Launcher 的历史版本。',
                },
            ],
        },
        notes: {
            'en-US': 'New versions are announced on this project\u2019s GitHub releases.',
            'zh-CN': '新版本只在本项目的 GitHub Releases 页面发布。',
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
                        'Rebranded the launcher as StarryIsle, including the installer and deep link.',
                    'zh-CN': '启动器更名为 StarryIsle，安装器与深链接同步更新。',
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
