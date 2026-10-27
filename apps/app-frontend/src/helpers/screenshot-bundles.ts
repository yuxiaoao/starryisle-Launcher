import { invoke } from '@tauri-apps/api/core'

export type BundleExportReport = {
    screenshotCount: number
    noteCount: number
}

export type BundleImportReport = {
    imported: number
    skipped: number
    overwritten: number
    renamed: number
    notesWritten: number
    notesSkipped: number
}

export type BundleConflictPolicy = 'skip' | 'overwrite' | 'rename'

export async function exportScreenshotBundle(
    instanceId: string,
    path: string,
): Promise<BundleExportReport> {
    return await invoke('plugin:instance|instance_export_screenshots_bundle', {
        instanceId,
        path,
    })
}

export async function importScreenshotBundle(
    instanceId: string,
    path: string,
    conflict: BundleConflictPolicy,
): Promise<BundleImportReport> {
    return await invoke('plugin:instance|instance_import_screenshots_bundle', {
        instanceId,
        path,
        conflict,
    })
}
