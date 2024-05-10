import { type PropsOf, Slot, component$ } from '@builder.io/qwik';

type DropdownOptionLabelProps = PropsOf<'span'>;

export const DropdownItemLabel = component$((props: DropdownOptionLabelProps) => {
    return (
        <span tabIndex={-1} {...props}>
            <Slot />
        </span>
    );
});