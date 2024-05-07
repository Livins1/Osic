
export type DisplayMeta = {
    name: string,
    deviceId: string,
    height: number,
    left: number,
    bottom: number,
    right: number,
    top: number,
    width: number,
}

export type DisplayBackgroundSelector = {
    path: string,
    ratio: boolean,
    ratioRange: number,
    ratioRangeUi: number,
    ratioValue: number,
    shuffle: boolean,
    wallpaperIndex: number
}

export type Display = {
    albumPath: string,
    deviceId: string,
    fit: string,
    image: string,
    imageHistory: string[],
    meta: DisplayMeta,
    mode: string,
    selector: DisplayBackgroundSelector
}


export type DisplayItems = {
    index: number,
    name: string,
    deviceId: string,
}

export type AppState = {
    displayList: Display[],
    displayItems: DisplayItems[],
}


