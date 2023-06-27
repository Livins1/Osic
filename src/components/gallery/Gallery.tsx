
import './Gallery.css'
import { IoAddCircleSharp, IoImagesSharp, IoRefreshCircleSharp, IoMenu } from 'solid-icons/io'
import { AiFillRightCircle, AiFillLeftCircle, AiFillDelete, AiFillFolderOpen } from 'solid-icons/ai'
import { BsChevronCompactLeft, BsChevronCompactRight } from 'solid-icons/bs'
import { open } from '@tauri-apps/api/dialog';

import { convertFileSrc } from "@tauri-apps/api/tauri";
import { appDataDir } from '@tauri-apps/api/path';
import { createEffect, createSignal, onMount, For, JSX, Show, createReaction } from 'solid-js';
import { emit, listen } from '@tauri-apps/api/event'

import { GalleryAddFolder, GalleryGetFolders, GalleryPreview, GalleryDelFolder, GalleryRescanFolder, PreviewPicutre } from './Invoke';
import { ShowInFolder } from '../../cmd/util';


const FolderAddBtn = () => {

    const onAddFolder = async () => {
        const selectedDir = await open({
            directory: true,
            multiple: false,
            defaultPath: await appDataDir()
        })
        if (selectedDir) {
            console.log(selectedDir)
            await GalleryAddFolder(selectedDir as string)
        }

    }


    return <button
        class='bg-slate-900 hover:bg-slate-600 text-white py-2 px-4 rounded'
        onClick={onAddFolder}
    >
        <div class='flex justify-center items-center gap-2'>
            <span>Folder</span>
            <IoAddCircleSharp size={24}></IoAddCircleSharp>
        </div>
    </button>
}



type PageSwtichGroupProps = {
    PageNum: () => Number
    TotalPageNum: () => Number
    SetPageNum: (arg: Number) => any
}
type PrewviewrPageButtonProps = {
    Icon: JSX.Element,
    OnClick: () => any
}
type PreviewOptBtnProps = {
    Icon: JSX.Element,
    OnClick: () => any,
    Title: string,
}


const PageSwtichGroup = (props: PageSwtichGroupProps) => {
    const { PageNum, SetPageNum, TotalPageNum } = props

    const onPrevPage = () => {
        SetPageNum(PageNum().valueOf() > 0 ? PageNum().valueOf() - 1 : 0)
    }
    const onNextPage = () => {
        SetPageNum(PageNum().valueOf() + 1 >= TotalPageNum().valueOf() ? PageNum() : PageNum().valueOf() + 1)
    }

    return <>
        <button
            class='bg-slate-900 hover:bg-slate-600 text-white py-2 px-4 rounded'
            onClick={onPrevPage}
        >
            <div class='flex justify-center items-center gap-2'>
                <AiFillLeftCircle size={24} />
                <span>Prev</span>
            </div>
        </button>
        <button
            class='bg-slate-900 hover:bg-slate-600 text-white py-2 px-4 rounded'
            onClick={onNextPage}
        >
            <div class='flex justify-center items-center gap-2'>
                <span>Next</span>
                <AiFillRightCircle size={24} />
            </div>
        </button>
    </>


}


export default function Gallery() {

    const PageShowFolderNum = 8
    const PreviewLimit = 5

    const [Folders, setFolders] = createSignal<Array<any>>([])
    // Show 8 item a Page this is Const
    const [pageNum, setPageNum] = createSignal<Number>(0)
    const [pageTotal, setPageTotal] = createSignal<Number>(1)
    const [folderIndex, setFolderIndex] = createSignal<Number>(0)
    const [previewPage, setPrewviewPage] = createSignal<Number>(0)
    const [previewImages, setPreviewImages] = createSignal<Array<any>>([])

    onMount(async () => {
        // Fetch Once onMount
        const folders = await GalleryGetFolders()
        if (folders) {
            setFolders(() => folders)
        }

        // register Listener
        const _ = await listen('update_folders', (event) => {
            setFolders(() => event.payload as Array<any>)
        })
    })



    createEffect(async () => {

        const res = await GalleryPreview(previewPage().valueOf(), PreviewLimit, folderIndex().valueOf())
        if (res) {
            setPreviewImages(() => res)
            console.log("PImages:", previewImages())
        }
    })




    const FolderItem = (props: any) => {

        const onClick = async () => {
            // Change Backend Selected Folder State 
            if (props.Folder.index >= 0) {
                setFolderIndex(() => props.Folder.index)
            } else {
                console.log("No Folder Index", props.Folder)
            }
            setPrewviewPage(() => 0)
            const res = await GalleryPreview(previewPage().valueOf(), PreviewLimit, props.Folder.index)
            if (res) {
                setPreviewImages(() => res)
                console.log("PImages:", previewImages())
            }
        }

        const onDelete = async (event: any) => {
            event.stopPropagation()
            await GalleryDelFolder(props.Folder.index)
        }
        const onRescan = async (event: any) => {
            event.stopPropagation()
            await GalleryRescanFolder(props.Folder.index)
        }

        const onMenu = async () => { }


        return <div class='flex flex-col justify-start bg-slate-900 text-white  max-w-full  w-full pl-1' onClick={onClick}>
            <div class='text-left text-ellipsis overflow-hidden truncate max-w-full m-2 font-medium text-lg'>
                {props.Folder.path}
            </div>
            <div class='flex justify-start items-center  w-full pl-2'>
                <IoImagesSharp size={18}></IoImagesSharp>
                <div class='ml-2 text-lg font-medium text-green-400 '>
                    {props.Folder.quanitity}
                </div>
                <div class='flex-1'></div>
                <button class='ml-3 rounded-none p-1 pl-4 pr-4' onClick={onRescan}>
                    <IoRefreshCircleSharp size={22}></IoRefreshCircleSharp>
                    {/* <AiFillDelete size={20}></AiFillDelete> */}
                </button>
                <button class='rounded-none p-1 pl-4 pr-4 ' onClick={onDelete}>
                    <AiFillDelete size={22}></AiFillDelete>
                </button>
                <button class='rounded-none p-1 pl-4 pr-4 ' >
                    <IoMenu size={22}></IoMenu>
                </button>
            </div>
        </div>
    }

    const FolderList = () => {
        const PageFolders = () => {
            console.log("PageFolders")
            const startIndex = pageNum().valueOf() * PageShowFolderNum
            return Folders().slice(startIndex, startIndex + PageShowFolderNum)
        }
        return <div class='grid grid-cols-2 grid-rows-4 grid-flow-dense gap-1' style={{ "max-height": "50vh" }}>
            <For each={PageFolders()} fallback={<div></div>} >
                {(item, index) => (
                    <div class='p-0'>
                        <FolderItem Folder={item} ></FolderItem>
                    </div>
                )
                }
            </For>
        </div>
    }

    const Previewer = () => {


        const [selectIndex, setSelected] = createSignal<Number>(-1)

        const trackSelected = createReaction(() => setSelected(-1))

        // if previewImages Changed, call trackSelected,  setSelected to -1
        trackSelected(() => previewImages())


        const onNext = async () => {
            if (previewImages().length < PreviewLimit) {
                console.log("???", previewImages().length)
                return
            }
            setPrewviewPage((prev) => prev.valueOf() + 1)
        }
        const onPrev = () => {
            setPrewviewPage((prev) => prev.valueOf() > 0 ? prev.valueOf() - 1 : 0)
        }

        const showInExplorer = async () => {
            const f = previewImages().at(selectIndex().valueOf())
            if (f.picture) {
                await ShowInFolder(f.picture.path)
            }
        }

        const previewAsWallpaper = async () => {
            console.log("preview")

            const f = previewImages().at(selectIndex().valueOf())
            if (f.picture) {
                await PreviewPicutre(f.picture.path)
            }


        }

        const onSelectedPicture = (index: number) => {

            if (index == selectIndex().valueOf()) {
                setSelected(-1)
            } else {
                setSelected(index)
            }
        }


        const PageButton = (props: PrewviewrPageButtonProps) => {
            const { Icon, OnClick } = props
            return <button
                class='bg-slate-900 hover:bg-slate-600 text-white mt-5 mb-5 m-2 px-2 rounded-none '
                onClick={OnClick}
            >
                <div class='flex justify-center items-center gap-2'>
                    {Icon}
                </div>
            </button>
        }

        const PreviewOptBtn = (props: PreviewOptBtnProps) => {
            const { Icon, OnClick, Title } = props
            return <button
                class='bg-slate-900 hover:bg-slate-600 text-white  m-2 px-2 rounded-none '
                onClick={OnClick}
            >
                <div class='flex justify-center items-center gap-2'>
                    {Icon}
                    <span>{Title}</span>
                </div>
            </button>
        }

        return <div class='flex flex-col'>
            <div class='flex flex-row'>
                <PreviewOptBtn Icon={<AiFillFolderOpen />} Title='Show in Exploer' OnClick={showInExplorer}></PreviewOptBtn>
                <PreviewOptBtn Icon={<AiFillFolderOpen />} Title='Preview' OnClick={previewAsWallpaper}></PreviewOptBtn>
            </div>
            <div class='flex flex-row'>
                <PageButton Icon={<BsChevronCompactLeft />} OnClick={onPrev}   ></PageButton>
                <div class='flex flex-row'>
                    <For each={previewImages()} fallback={<div></div>} >
                        {(item, index) => (
                            <div class='h-48 flex  items-center justify-center p-1  ' classList={{
                                'hover:bg-slate-700': selectIndex() != index(),
                                'bg-slate-600': selectIndex() == index(),

                            }} onClick={() => onSelectedPicture(index())}>
                                <div class='h-40 overflow-hidden items-center self-center flex'>
                                    <img src={item.thumbnail}></img>
                                </div>
                            </div>
                        )
                        }
                    </For>
                </div >
                <div class='flex-1'></div>
                <PageButton Icon={<BsChevronCompactRight />} OnClick={onNext}></PageButton>
            </div >
        </div>

    }

    return (
        <div class="Component-Container">
            <div class='container flex flex-col'
            >
                <div class='flex justify-end m-2 gap-2'>
                    <PageSwtichGroup TotalPageNum={pageTotal} SetPageNum={setPageNum} PageNum={pageNum}></PageSwtichGroup>
                    <FolderAddBtn></FolderAddBtn>
                </div>
                <div class=''>
                    <FolderList></FolderList>
                </div>
                <div class='flex-1'></div>
                <Show when={previewImages().length > 0}>
                    <Previewer></Previewer>
                </Show>

            </div>
        </div>
    )

}