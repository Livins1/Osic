import { useTask$, Slot, component$, type PropsOf, useSignal, $, useContext, useComputed$, useContextProvider } from "@builder.io/qwik";
import DropdownContextId, { type DropdownItemContext, DropdownItemContextId } from "./context";
import { useSelect } from "./use-select";
import { isBrowser, isServer } from "@builder.io/qwik/build";

export type DropdownItemProps = PropsOf<'li'> & {
    _index?: number;
    disabled?: boolean;
    value?: string;
};


export const DropdownItem = component$<DropdownItemProps>((props) => {

    const { _index, disabled, ...rest } = props
    const context = useContext(DropdownContextId);
    const itemRef = useSignal<HTMLLIElement>()
    const localIndexSig = useSignal<number | null>(null);
    const itemId = `${context.localId}-${_index}`;

    const { selectionManager$ } = useSelect();

    const isSelectedSig = useComputed$(() => {

        const index = _index ?? null;
        return !disabled && context.selectedIndexSetSig.value.has(index!);
    });

    const isHighlightedSig = useComputed$(() => {
        return !disabled && context.highlightedIndexSig.value === _index;
    });

    useTask$(function getIndexTask() {
        if (_index === undefined)
            throw Error('Select component item cannot find its proper index.');

        localIndexSig.value = _index;
    });

    useTask$(function scrollableTask({ track, cleanup }) {
        track(() => context.highlightedIndexSig.value);

        if (isServer) return;

        let observer: IntersectionObserver;

        const checkVisibility = (entries: IntersectionObserverEntry[]) => {
            const [entry] = entries;

            // if the is not visible, scroll it into view
            if (isHighlightedSig.value && !entry.isIntersecting) {
                itemRef.value?.scrollIntoView(context.scrollOptions);
            }
        };

        cleanup(() => observer?.disconnect());

        if (isBrowser) {
            observer = new IntersectionObserver(checkVisibility, {
                root: context.listboxRef.value,
                threshold: 1.0,
            });

            if (itemRef.value) {
                observer.observe(itemRef.value);
            }
        }
    });

    const handleClick$ = $(async () => {
        if (disabled) return;
        console.log("handleClick")
        await selectionManager$(localIndexSig.value, 'add');
        context.isListboxOpenSig.value = false
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
            id={itemId}
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
