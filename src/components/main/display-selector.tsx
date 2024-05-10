import { useComputed$, component$, useContext, useStyles$, useSignal, $ } from "@builder.io/qwik";
import { Dropdown } from "../qwik-ui";

import { AppContextId } from "~/cmd/context";



export const DisplaySelector = component$(() => {

    const state = useContext(AppContextId)

    useStyles$(styles);

    const items = useComputed$(() => state.displayItems)
    const selected = useSignal<string>('')


    return (
        <Dropdown.Root bind:value={selected}>
            <Dropdown.Trigger class="select-trigger">
                <Dropdown.DisplayText placeholder="Dropdown an option" />
            </Dropdown.Trigger>
            <Dropdown.Popover class="select-popover">
                <Dropdown.ListBox class="select-listbox">
                    {items.value.map((item, index) => (
                        <Dropdown.Item key={index}>
                            <Dropdown.ItemLabel>{item.label.toString()}</Dropdown.ItemLabel>
                        </Dropdown.Item>
                    ))}
                </Dropdown.ListBox>
            </Dropdown.Popover>
        </Dropdown.Root >

    )

})

// internal
import styles from './display-selector.css?inline';

