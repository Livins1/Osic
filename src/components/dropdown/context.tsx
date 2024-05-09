
import { createContextId, type Signal } from "@builder.io/qwik"

import { type TItemsMap } from "./root";

const DropdownContextId = createContextId<DropdownContext>('DropdownMenu')


export default DropdownContextId;


export type DropdownContext = {


    // refs
    triggerRef: Signal<HTMLButtonElement | undefined>;
    popoverRef: Signal<HTMLElement | undefined>;
    listboxRef: Signal<HTMLUListElement | undefined>;
    groupRef: Signal<HTMLDivElement | undefined>;
    labelRef: Signal<HTMLDivElement | undefined>;

    // core state
    itemsMapSig: Readonly<Signal<TItemsMap>>;
    selectedIndexSetSig: Signal<Set<number>>;
    highlightedIndexSig: Signal<number | null>;
    currDisplayValueSig: Signal<string | string[] | undefined>;
    isListboxOpenSig: Signal<boolean>;
    localId: string;

    // user configurable
    scrollOptions?: ScrollIntoViewOptions;

    disabled?: boolean,
    required?: boolean,
    name?: string,
    loop?: boolean
}

export const groupContextId = createContextId<GroupContext>('Dropdown-Group');

export type GroupContext = {
    groupLabelId: string;
};

export const DropdownItemContextId = createContextId<DropdownItemContext>('Dropdown-Option');

export type DropdownItemContext = {
    isSelectedSig: Signal<boolean>;
};


