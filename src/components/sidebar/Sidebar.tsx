import { A } from '@solidjs/router'
import { mergeProps } from 'solid-js'
import './Sidebar.css'

type SideBarItemProps = {
    path: string,
    name: string
}


function SideBarItem(props: SideBarItemProps) {
    const merged = mergeProps({ path: '/', name: "EmptyItemName" }, props)
    return <div class='SideBarItem'>
        <A href={merged.path} activeClass='Active' inactiveClass='InActive'>{merged.name}</A>
    </div>
}

export default function SideBar() {
    return <div class="SideBar">
        <SideBarItem path='/Gallery' name='Gallery'></SideBarItem>
        <SideBarItem path='/Monitor' name='Monitors'></SideBarItem>
        <SideBarItem path='/About' name='About'></SideBarItem>
    </div>
}