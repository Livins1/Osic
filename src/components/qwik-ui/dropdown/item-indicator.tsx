import { Slot, component$, useContext } from '@builder.io/qwik';
import { DropdownItemContextId } from './context';

export const DropdownItemIndicator = component$(() => {
    const selectContext = useContext(DropdownItemContextId);

    return <>{selectContext.isSelectedSig.value && <Slot />}</>;
});