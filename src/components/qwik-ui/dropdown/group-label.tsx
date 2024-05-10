
import { type PropsOf, Slot, component$, useContext } from '@builder.io/qwik';
import { groupContextId } from './context';

type SelectLabelProps = PropsOf<'li'>;

export const DropdownGroupLabel = component$<SelectLabelProps>((props) => {
    const groupContext = useContext(groupContextId);

    return (
        <li id={groupContext.groupLabelId} {...props}>
            <Slot />
        </li>
    );
});