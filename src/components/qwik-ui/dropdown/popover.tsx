
import {
    component$,
    useContext,
    Slot,
    useTask$,
    type PropsOf,
    useSignal,
} from '@builder.io/qwik';
import { usePopover } from '../popover/trigger';
import { PopoverPanel } from '../popover/panel';

// import SelectContextId from './select-context';
import DropdownContextId from './context';
import { PopoverRoot } from '../popover/root';
import { isServer } from '@builder.io/qwik/build';

export const DropdownPopover = component$<PropsOf<typeof PopoverRoot>>((props) => {
    const context = useContext(DropdownContextId);
    const { showPopover, hidePopover } = usePopover(context.localId);

    const { floating, flip, hover, gutter, ...rest } = props;
    const initialLoadSig = useSignal<boolean>(true);

    useTask$(async ({ track }) => {
        track(() => context.isListboxOpenSig.value);

        if (isServer) return;

        if (!initialLoadSig.value) {
            if (context.isListboxOpenSig.value) {
                await showPopover();
            } else {
                await hidePopover();
            }
        }
    });

    useTask$(() => {
        initialLoadSig.value = false;
    });

    return (
        <PopoverRoot
            floating={floating}
            flip={flip}
            hover={hover}
            gutter={gutter}
            bind:anchor={context.triggerRef}
            manual
            id={context.localId}
        >
            <PopoverPanel
                data-open={context.isListboxOpenSig.value ? '' : undefined}
                data-closed={!context.isListboxOpenSig.value ? '' : undefined}
                {...rest}
            >
                <Slot />
            </PopoverPanel>
        </PopoverRoot>
    );
});