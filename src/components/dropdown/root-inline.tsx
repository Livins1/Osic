import type {  JSXNode, Component } from '@builder.io/qwik';
import { DropdownLabel as InternalDropdownLabel } from "./label";
import { DropdownItemLabel as InternalItemLabel } from "./item-label";
import { DropdownItem as InternalItem } from "./item";
import { DropdwonImpl, type DropdownProps } from "./root";



type InlineCompProps = {
    selectLabelComponent?: typeof InternalDropdownLabel;
    selectItemComponent?: typeof InternalItem;
    selectItemLabelComponent?: typeof InternalItemLabel;
};


// just learning qwik with qwik-ui
// https://github.com/qwikifiers/qwik-ui/blob/main/packages/kit-headless/src/components/select/select-inline.tsx
export const DropdownRoot: Component<DropdownProps & InlineCompProps> = (props: InlineCompProps & DropdownProps) => {
    const {
        children: myChildren,
        selectLabelComponent: UserLabel,
        selectItemComponent: UserItem,
        selectItemLabelComponent: UserItemLabel,
        ...rest
    } = props

    const SelectLabel = UserLabel ?? InternalDropdownLabel;
    const SelectItem = UserItem ?? InternalItem;
    const SelectItemLabel = UserItemLabel ?? InternalItemLabel;

    const itemsMap = new Map()

    let currItemIndex = 0
    let isItemDisabled = false;
    let givenItemValue = null;
    let valuePropIndex = null;
    let isLabelNeeded = false;

    const childrenToProcess = (
        Array.isArray(myChildren) ? [...myChildren] : [myChildren]
    ) as Array<JSXNode>;

    while (childrenToProcess.length) {
        const child = childrenToProcess.shift();

        if (!child) {
            continue;
        }

        if (Array.isArray(child)) {
            childrenToProcess.unshift(...child);
            continue;
        }

        switch (child.type) {
            case SelectLabel: {
                isLabelNeeded = true;
                break;
            }

            case SelectItem: {
                // get the index of the current option
                child.props._index = currItemIndex;

                isItemDisabled = child.props.disabled === true;

                if (child.props.value) {
                    givenItemValue = child.props.value;
                }

                // the default case isn't handled here, so we need to process the children to get to the label component
                if (child.props.children) {
                    const childChildren = Array.isArray(child.props.children)
                        ? [...child.props.children]
                        : [child.props.children];
                    childrenToProcess.unshift(...childChildren);
                }

                break;
            }

            case SelectItemLabel: {
                const displayValue = child.props.children as string;

                // distinct value, or the display value is the same as the value
                const value = (givenItemValue !== null ? givenItemValue : displayValue) as string;

                itemsMap.set(currItemIndex, { value, displayValue, disabled: isItemDisabled });

                // if (props.value && props.multiple) {
                //     throw new Error(
                //         `Qwik UI: When in multiple selection mode, the value prop is disabled. Use the bind:value prop's initial signal value instead.`,
                //     );
                // }

                // if the current option value is equal to the initial value
                if (value === props.value) {
                    // minus one because it is incremented already in SelectOption
                    valuePropIndex = currItemIndex;
                }

                const isString = typeof child.props.children === 'string';

                if (!isString) {
                    throw new Error(
                        `Qwik UI: select item label passed was not a string. It was a ${typeof child
                            .props.children}.`,
                    );
                }

                // increment after processing children
                currItemIndex++;

                break;
            }

            default: {
                if (child) {
                    const anyChildren = Array.isArray(child.children)
                        ? [...child.children]
                        : [child.children];
                    childrenToProcess.unshift(...(anyChildren as JSXNode[]));
                }

                break;
            }
        }
    }

    return (
        <DropdwonImpl
            {...rest}
            _label={isLabelNeeded}
            _valuePropsIndex={valuePropIndex}
            _itemsMap={itemsMap}
        >
            {props.children}
        </DropdwonImpl>
    );






}
