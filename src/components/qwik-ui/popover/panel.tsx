
import { component$, type PropsOf, Slot, useContext } from '@builder.io/qwik';
import { FloatingPopover } from './floating';
import { PopoverPanelImpl } from './panel-impl';
import { popoverContextId } from './context';

// TODO: improve the type so that it only includes FloatingProps when floating is true.

/* This component determines whether the popover needs floating behavior, a common example where it doesn't, would be a toast. */
export const PopoverPanel = component$((props: PropsOf<'div'>) => {
    const context = useContext(popoverContextId);

    if (context.floating) {
        return (
            <FloatingPopover data-floating {...props}>
                <Slot />
            </FloatingPopover>
        );
    }

    return (
        <PopoverPanelImpl {...props}>
            <Slot />
        </PopoverPanelImpl>
    );
});