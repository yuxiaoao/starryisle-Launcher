import { invoke } from '@tauri-apps/api/core'

export type StatsDay = {
    date: string
    seconds: number
    topInstance: string | null
}

export type StatsInstanceTotal = {
    name: string
    seconds: number
}

export type StatsTimelineEntry = {
    id: string
    title: string
    iconPath: string | null
    instanceName: string
    worldName: string
    completedAt: number | null
}

export type StatsOverview = {
    totalSeconds: number
    instanceCount: number
    screenshotCount: number
    achievementTotal: number
    days: StatsDay[]
    instances: StatsInstanceTotal[]
    timeline: StatsTimelineEntry[]
    years: number[]
}

export async function getStatsOverview(year?: number, force = false): Promise<StatsOverview> {
    return await invoke('plugin:instance|stats_overview', {
        year: year ?? null,
        force,
    })
}
