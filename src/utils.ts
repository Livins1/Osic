
import { invoke } from '@tauri-apps/api'

export const Greet = () => {
    invoke('greet', { name: "Kiss" }).then((res) => {
        console.log(res)
    })
}