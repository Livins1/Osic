
import {
    type PropsOf,
    Slot,
    component$,
    useContext,
    useContextProvider,
} from '@builder.io/qwik';

import DropdownContextId, { groupContextId } from './context';

type SelectGroupProps = PropsOf<'div'>;

export const DropdownGroup = component$<SelectGroupProps>((props) => {
    const context = useContext(DropdownContextId);
    const groupLabelId = `${context.localId}-group-label`;

    const groupContext = {
        groupLabelId,
    };

    useContextProvider(groupContextId, groupContext);

    return (
        <div aria-labelledby={groupLabelId} role="group" {...props} ref={context.groupRef}>
            <Slot />
        </div>
    );
});