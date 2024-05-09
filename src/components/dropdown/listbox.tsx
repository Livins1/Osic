import { Slot, useTask$, $,type PropsOf, component$, useContext } from "@builder.io/qwik";
import DropdownContextId from "./context";
import { isServer } from '@builder.io/qwik/build';

type DropdownListboxProps = PropsOf<'ul'>


export const DropdownListbox = component$<DropdownListboxProps>((props) => {

    const context = useContext(DropdownContextId);


    const listboxId = `${context.localId}-listbox`

    const isOutside = $((rect: DOMRect, x: number, y: number) => {
        return x < rect.left || x > rect.right || y < rect.top || y > rect.bottom;
    })

    const handleDismiss$ = $(async (e: PointerEvent) => {
        if (!context.isListboxOpenSig.value) { return }
        if (!context.listboxRef.value || !context.triggerRef.value) { return }

        const listboxRect = context.listboxRef.value.getBoundingClientRect();
        const triggerRect = context.triggerRef.value.getBoundingClientRect();
        const { clientX, clientY } = e;

        const isOutsideListbox = await isOutside(listboxRect, clientX, clientY);
        const isOutsideTrigger = await isOutside(triggerRect, clientX, clientY);

        if (isOutsideListbox && isOutsideTrigger) {
            context.isListboxOpenSig.value = false;
        }

    })

    // Dismiss code should only matter when the listbox is open
    useTask$(({ track, cleanup }) => {
        track(() => context.isListboxOpenSig.value);

        if (isServer) return;

        if (context.isListboxOpenSig.value) {
            window.addEventListener('pointerdown', handleDismiss$);
        }

        cleanup(() => {
            window.removeEventListener('pointerdown', handleDismiss$);
        });
    });

    return (
        <ul
            {...props}
            id={listboxId}
            role="listbox"
            ref={context.listboxRef}
            data-open={context.isListboxOpenSig.value ? '' : undefined}
            data-closed={!context.isListboxOpenSig.value ? '' : undefined}
        >
            <Slot />
        </ul>
    )

})
