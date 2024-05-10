
import { type PropsOf, component$, useContext } from '@builder.io/qwik';
import { popoverContextId } from './context';

export const PopoverPanelArrow = component$((props: PropsOf<'div'>) => {
    const context = useContext(popoverContextId);

    return (
        <div
            ref={context.arrowRef}
            style={{ position: 'absolute', background: 'red' }}
            {...props}
        />
    );
});