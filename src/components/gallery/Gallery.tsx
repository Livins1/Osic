
import './Gallery.css'
import { IoAddCircleSharp } from 'solid-icons/io'
import { AiFillRightCircle, AiFillLeftCircle } from 'solid-icons/ai'
import { BsChevronCompactLeft, BsChevronCompactRight } from 'solid-icons/bs'
import { open } from '@tauri-apps/api/dialog';

import { convertFileSrc } from "@tauri-apps/api/tauri";
import { appDataDir } from '@tauri-apps/api/path';
import { createEffect, createSignal, onMount, For, JSX, Show } from 'solid-js';
import { emit, listen } from '@tauri-apps/api/event'

import { GalleryAddFolder, GalleryGetFolders, GalleryPreview } from './Invoke';


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


const PageSwtichGroup = (props: PageSwtichGroupProps) => {
    const { PageNum, SetPageNum, TotalPageNum } = props

    const onPrevPage = () => {
        SetPageNum(PageNum().valueOf() > 0 ? PageNum().valueOf() - 1 : 0)
    }
    const onNextPage = () => {
        SetPageNum(PageNum() >= TotalPageNum() ? TotalPageNum() : PageNum().valueOf() + 1)
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
    const PreviewLimit = 6

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


        return <button class='bg-slate-900 text-white  max-w-full  w-full' onClick={onClick}>
            <div class='text-left text-ellipsis overflow-hidden truncate max-w-full'>
                {props.Folder.path}
            </div>
        </button>
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
                    <div class='' >
                        <div class='p-0'>
                            <FolderItem Folder={item} ></FolderItem>
                        </div>
                    </div>
                )
                }
            </For>
        </div>
    }

    const Previewer = () => {


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


        const PageButton = (props: PrewviewrPageButtonProps) => {
            const { Icon, OnClick } = props
            return <button
                class='bg-slate-900 hover:bg-slate-600 text-white py-2 px-4 rounded'
                onClick={OnClick}
            >
                <div class='flex justify-center items-center gap-2'>
                    {Icon}
                </div>
            </button>
        }


        return <div class='flex flex-row'>
            <PageButton Icon={<BsChevronCompactLeft />} OnClick={onPrev}   ></PageButton>
            <div class='flex flex-row'>
                <For each={previewImages()} fallback={<div></div>} >
                    {(item, index) => (
                        <div class='h-2 flex  items-center justify-center  p-1'>
                            <div class='p-0'>
                                {/* <img class='h-36' src={convertFileSrc(item.picture.path)}></img> */}
                                <img class='h-36' src={item.thumbnail}></img>
                            </div>
                        </div>
                    )
                    }
                </For>
            </div>
            <PageButton Icon={<BsChevronCompactRight />} OnClick={onNext}></PageButton>
        </div>
    }

    return (
        <div class="Component-Container">
            <div class='container mx-auto flex flex-col'
            >
                <div class='flex flex-col'>
                    <div class='flex justify-end m-2 gap-2'>
                        <PageSwtichGroup TotalPageNum={pageTotal} SetPageNum={setPageNum} PageNum={pageNum}></PageSwtichGroup>
                        <FolderAddBtn></FolderAddBtn>
                    </div>
                    <FolderList></FolderList>
                    <Show when={previewImages().length > 0}>
                        <Previewer></Previewer>
                    </Show>

                </div>

            </div>
        </div>
    )

}