import { Slot, $, type PropsOf, component$, sync$, useContext } from "@builder.io/qwik";
import DropdownContextId from "./context";
import { useSelect } from "./use-select";


type DropdownTriggerProps = PropsOf<'button'>

export const DropdownTrigger = component$<DropdownTriggerProps>((props) => {
    const context = useContext(DropdownContextId)

    const { getNextEnabledItemIndex$ } = useSelect()
    const labelId = `${context.localId}-label`
    const descriptionId = `${context.localId}-description`


    const handleClickSync$ = sync$((e: MouseEvent) => {
        e.preventDefault()
    })
    const handleClick$ = $(() => {
        context.isListboxOpenSig.value = !context.isListboxOpenSig.value
    })

    const handleKeyDownSync$ = sync$((e: KeyboardEvent) => {
        const keys = [
            'ArrowUp',
            'ArrowDown',
            'ArrowRight',
            'ArrowLeft',
            'Home',
            'End',
            'PageDown',
            'PageUp',
            'Enter',
            ' ',
        ];
        if (keys.includes(e.key)) {
            e.preventDefault();
        }
    })



    const handleKeyDown$ = $(async (e: KeyboardEvent) => {

        if (!context.itemsMapSig.value) return


        /** When initially opening the listbox, we want to grab the first enabled option index */
        if (context.highlightedIndexSig.value === null) {
            context.highlightedIndexSig.value = await getNextEnabledItemIndex$(-1);
            return;
        }
    })


    return (
        <button
            {...props}
            id={`${context.localId}-trigger`}
            ref={context.triggerRef}
            onClick$={[handleClickSync$, handleClick$, props.onClick$]}
            onKeyDown$={[handleKeyDownSync$, handleKeyDown$, props.onKeyDown$]}
            data-open={context.isListboxOpenSig.value ? '' : undefined}
            data-closed={!context.isListboxOpenSig.value ? '' : undefined}
            data-disabled={context.disabled ? '' : undefined}
            aria-expanded={context.isListboxOpenSig.value}
            aria-labelledby={labelId}
            aria-describedby={descriptionId}
            disabled={context.disabled}
            preventdefault:blur
        >
            <Slot />
        </button>
    );

}) 