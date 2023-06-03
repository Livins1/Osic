

import { invoke } from '@tauri-apps/api'

export const GalleryAddFolder = async (path: string) => {
    await invoke('add_folder', { path: path })
}

export const GalleryGetFolders = async (): Promise<Array<Object>> => {
    const res = await invoke('get_folders', {})
    console.log(res)
    return res as Array<Object>
}

export const GalleryPreview = async (page :number, size: number, folderIndex: number): Promise<Array<any>> => {
    const res = await invoke('preview', {page, size, folderIndex})
    console.log(res)
    return res as Array<any>
}