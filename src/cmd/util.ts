
import { invoke } from '@tauri-apps/api'


export const ShowInFolder = async (path: string) => {
    await invoke('explorer_file', { path })
}