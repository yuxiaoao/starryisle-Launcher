import { invoke } from '@tauri-apps/api/core'

export type PlayerRef = {
    id: string
    name: string | null
}

export type WorldSummary = {
    name: string
    lastPlayed: number | null
    players: PlayerRef[]
    hasAdvancements: boolean
}

export type CompletedAdvancement = {
    id: string
    completedAt: number | null
}

export type WorldAdvancements = {
    player: string | null
    players: PlayerRef[]
    completed: CompletedAdvancement[]
}

export type AdvancementDef = {
    id: string
    title: string
    description: string
    category: string
    iconItem: string | null
    iconPath: string | null
    hidden: boolean
}

export async function listWorlds(instanceId: string): Promise<WorldSummary[]> {
    return await invoke('plugin:instance|instance_list_worlds', { instanceId })
}

export async function getWorldAdvancements(
    instanceId: string,
    world: string,
    player?: string | null,
): Promise<WorldAdvancements> {
    return await invoke('plugin:instance|instance_world_advancements', {
        instanceId,
        world,
        player: player ?? null,
    })
}

export async function getAdvancementCatalog(instanceId?: string): Promise<AdvancementDef[]> {
    return await invoke('plugin:instance|instance_advancement_catalog', {
        instanceId: instanceId ?? null,
    })
}
