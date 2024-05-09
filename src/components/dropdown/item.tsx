import { useTask$, Slot, component$, type PropsOf, useSignal, $, useContext, useComputed$, useContextProvider } from "@builder.io/qwik";
import DropdownContextId, { type DropdownItemContext, DropdownItemContextId } from "./context";

export type DropdownItemProps = PropsOf<'li'> & {
    index?: number;
    disabled?: boolean;
    value?: string;
};


export const DropdownItem = component$<DropdownItemProps>((props) => {

    const { index, disabled, ...rest } = props
    const context = useContext(DropdownContextId);
    const itemRef = useSignal<HTMLLIElement>
    const localIndexSig = useSignal<number | null>(null);

    const isSelectedSig = useComputed$(() => {
        // const index = index ?? null;
        return !disabled && context.selectedIndexSetSig.value.has(index!);
    });

    const isHighlightedSig = useComputed$(() => {
        return !disabled && context.highlightedIndexSig.value === _index;
    });

    useTask$(function getIndexTask() {
        if (index === undefined)
            throw Error('Select component item cannot find its proper index.');

        localIndexSig.value = index;
    });

    const handleClick$ = $(async () => {
        if (disabled) return;

        // await selectionManager$(localIndexSig.value, 'add');
    });

    const handlePointerOver$ = $(() => {
        if (disabled) return;

        if (localIndexSig.value !== null) {
            context.highlightedIndexSig.value = localIndexSig.value;
        }
    });

    const dropdownContext: DropdownItemContext = {
        isSelectedSig
    }


    useContextProvider(DropdownItemContextId, dropdownContext)


    return (
        <li
            {...rest}
            onClick$={[handleClick$, props.onClick$]}
            onPointerOver$={[handlePointerOver$, props.onPointerOver$]}
            ref={itemRef}
            tabIndex={-1}
            aria-selected={isSelectedSig.value}
            aria-disabled={disabled === true ? 'true' : 'false'}
            data-selected={isSelectedSig.value ? '' : undefined}
            data-highlighted={isHighlightedSig.value ? '' : undefined}
            data-disabled={disabled ? '' : undefined}
            data-item
            role="option"
        >
            <Slot />
        </li>
    )

})
