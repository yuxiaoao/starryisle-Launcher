import { invoke } from '@tauri-apps/api/core'

export const MAX_SCREENSHOT_NOTE_LENGTH = 30

export type ScreenshotNote = {
    instanceId: string
    fileName: string
    note: string
    updatedAt: number
}

export function screenshotNoteKey(instanceId: string, fileName: string): string {
    return `${instanceId}\u0000${fileName}`
}

export async function getScreenshotNotes(instanceId?: string): Promise<ScreenshotNote[]> {
    return await invoke('plugin:instance|instance_list_screenshot_notes', {
        instanceId: instanceId ?? null,
    })
}

export async function setScreenshotNote(
    instanceId: string,
    fileName: string,
    note: string,
): Promise<void> {
    await invoke('plugin:instance|instance_set_screenshot_note', {
        instanceId,
        fileName,
        note,
    })
}
