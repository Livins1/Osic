import { useComputed$, component$, useContext, useStyles$, useSignal, $, useVisibleTask$ } from "@builder.io/qwik";
import { Dropdown } from "../../qwik-ui";

import { AppContextId } from "~/cmd/context";


export const DisplaySelector = component$(() => {

    const state = useContext(AppContextId)

    useStyles$(styles);

    const items = useComputed$(() => state.displayItems)
    const selected = useSignal<string>('')


    useVisibleTask$(({ track }) => {
        track(() => items.value)
        selected.value = items.value[0]?.label
    })


    return (
        <div class="">
            <Dropdown.Root bind:value={selected} class=" bg-white/5  w-full align-middle block  rounded-lg border-none text-sm/6 text-white ">
                <Dropdown.Trigger class="select-trigger">
                    <Dropdown.DisplayText class="relative w-full text-left px-3" placeholder="Dropdown an option" />
                </Dropdown.Trigger>
                <Dropdown.Popover arrow={true} class="select-popover w-full  bg-white/5  mt-1 border-none text-sm/6 text-white rounded-lg border-nonetext-sm/6 text-center " >
                    <Dropdown.ListBox class="select-listbox" >
                        {items.value.map((item, index) => (
                            <Dropdown.Item key={index} class="text-left px-3 my-2">
                                <Dropdown.ItemLabel class="text-left" >{item.label.toString()}</Dropdown.ItemLabel>
                            </Dropdown.Item>
                        ))}
                    </Dropdown.ListBox>
                </Dropdown.Popover>
            </Dropdown.Root >
        </div >
    )

})

// internal
import styles from './display-selector.css?inline';

